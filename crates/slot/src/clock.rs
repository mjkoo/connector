#[cfg(feature = "test-support")]
use std::sync::{Arc, Mutex};
use std::sync::{Condvar, MutexGuard};
use std::time::{Duration, Instant};

/// Where the emulator thread reads the time and waits for it to pass.
///
/// `Real` is the wall clock. `Manual` is a clock a test moves by hand, so pacing
/// can be checked frame for frame however slowly the machine runs the thread.
#[derive(Clone)]
pub(crate) enum EmuClock {
    Real,
    #[cfg(feature = "test-support")]
    Manual(Arc<ManualClock>),
}

impl EmuClock {
    pub(crate) fn now(&self) -> Instant {
        match self {
            EmuClock::Real => Instant::now(),
            #[cfg(feature = "test-support")]
            EmuClock::Manual(m) => m.now(),
        }
    }

    pub(crate) fn since(&self, earlier: Instant) -> Duration {
        self.now().saturating_duration_since(earlier)
    }

    pub(crate) fn sleep(&self, d: Duration) {
        match self {
            EmuClock::Real => std::thread::sleep(d),
            #[cfg(feature = "test-support")]
            EmuClock::Manual(m) => m.sleep(d),
        }
    }

    /// Lets a manual clock run without bound, so its thread can finish.
    pub(crate) fn release(&self) {
        #[cfg(feature = "test-support")]
        if let EmuClock::Manual(m) = self {
            m.release();
        }
    }

    /// Waits on `cv` for at most `d`. Like `Condvar::wait_timeout`, it may return
    /// early, so the caller re-checks both its condition and the time.
    pub(crate) fn wait_timeout<'a, T>(
        &self,
        cv: &Condvar,
        guard: MutexGuard<'a, T>,
        d: Duration,
    ) -> MutexGuard<'a, T> {
        match self {
            EmuClock::Real => match cv.wait_timeout(guard, d) {
                Ok((g, _)) => g,
                Err(e) => e.into_inner().0,
            },
            #[cfg(feature = "test-support")]
            EmuClock::Manual(m) => m.wait_timeout(cv, guard, d),
        }
    }
}

/// A clock that only moves when told to.
///
/// The test sets a horizon with `run_for`. The thread on the clock runs freely
/// until it waits for a moment past the horizon, and time jumps to whatever it
/// waits for up to there. Work it does moves the clock with `advance`. `run_for`
/// returns once the thread is parked waiting beyond the new horizon, so whatever
/// the test reads afterwards is settled.
///
/// Waits end exactly when asked unless the test makes them late with
/// `oversleep_next` or `set_wake_latency`, the way a busy scheduler would.
#[cfg(feature = "test-support")]
pub struct ManualClock {
    base: Instant,
    state: Mutex<State>,
    moved: Condvar,
}

#[cfg(feature = "test-support")]
struct State {
    now: Duration,
    horizon: Duration,
    parked: bool,
    oversleep: Duration,
    latency: Duration,
    waiting_for: Option<Duration>,
}

/// How long a parked condvar wait sleeps for real between checks of the horizon.
/// A notify on the condvar still wakes it at once.
#[cfg(feature = "test-support")]
const POLL: Duration = Duration::from_millis(1);

/// How long `run_for` waits for the thread to park before it gives up.
#[cfg(feature = "test-support")]
const SETTLE: Duration = Duration::from_secs(10);

#[cfg(feature = "test-support")]
impl ManualClock {
    pub fn new() -> Arc<ManualClock> {
        Arc::new(ManualClock {
            base: Instant::now(),
            state: Mutex::new(State {
                now: Duration::ZERO,
                horizon: Duration::ZERO,
                parked: false,
                oversleep: Duration::ZERO,
                latency: Duration::ZERO,
                waiting_for: None,
            }),
            moved: Condvar::new(),
        })
    }

    fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn now(&self) -> Instant {
        self.base + self.lock().now
    }

    /// Time spent working: moves the clock whatever the horizon.
    pub fn advance(&self, d: Duration) {
        self.lock().now += d;
    }

    /// Makes the next wait that ends wake `d` late, on top of any latency.
    pub fn oversleep_next(&self, d: Duration) {
        self.lock().oversleep = d;
    }

    /// Makes every wait that ends wake `d` late.
    pub fn set_wake_latency(&self, d: Duration) {
        self.lock().latency = d;
    }

    /// Lets the clock run `d` further, and returns once the thread on it has
    /// parked waiting for a moment past that.
    ///
    /// A thread parked in a condvar wait still answers a notify on that condvar,
    /// such as a display tick or a state request, without the clock moving.
    #[track_caller]
    pub fn run_for(&self, d: Duration) {
        let mut s = self.lock();
        s.horizon = s.horizon.saturating_add(d);
        s.parked = false;
        self.moved.notify_all();
        let give_up = Instant::now() + SETTLE;
        while !s.parked {
            let Some(left) = give_up.checked_duration_since(Instant::now()) else {
                panic!(
                    "the thread on the manual clock never settled: the clock is at {:?} of a \
                     {:?} horizon, and its last wait was for {:?}",
                    s.now, s.horizon, s.waiting_for
                );
            };
            s = self
                .moved
                .wait_timeout(s, left)
                .unwrap_or_else(|e| e.into_inner())
                .0;
        }
    }

    /// Lets the clock run without bound, so a thread parked on it can finish.
    pub fn release(&self) {
        self.lock().horizon = Duration::MAX;
        self.moved.notify_all();
    }

    /// Takes the clock to `at`, plus any lateness, if the horizon allows; true
    /// when it did. Like work, lateness may carry the clock past the horizon.
    fn reach(&self, s: &mut State, at: Duration) -> bool {
        s.waiting_for = Some(at);
        if at <= s.horizon {
            let late = s.latency.saturating_add(std::mem::take(&mut s.oversleep));
            s.now = s.now.max(at.saturating_add(late));
            return true;
        }
        if !s.parked {
            s.parked = true;
            self.moved.notify_all();
        }
        false
    }

    fn sleep(&self, d: Duration) {
        let mut s = self.lock();
        let at = s.now.saturating_add(d);
        while !self.reach(&mut s, at) {
            s = self.moved.wait(s).unwrap_or_else(|e| e.into_inner());
        }
    }

    fn wait_timeout<'a, T>(
        &self,
        cv: &Condvar,
        guard: MutexGuard<'a, T>,
        d: Duration,
    ) -> MutexGuard<'a, T> {
        {
            let mut s = self.lock();
            let at = s.now.saturating_add(d);
            if self.reach(&mut s, at) {
                return guard;
            }
        }
        match cv.wait_timeout(guard, POLL) {
            Ok((g, _)) => g,
            Err(e) => e.into_inner().0,
        }
    }
}
