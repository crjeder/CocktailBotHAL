# Cleaning: a mode change, not a job
Idle ─────(admin: start_cleaning)──► Cleaning ──(power off)──► Off
Provisioning ─(admin: start_cleaning)──►  │
                                          │ clears job queue on entry
                                          │
                                  (only exit: power-off)

Cleaning → Idle is not valid. You power off, physically swap containers back, power on → SelfTest → Idle. This means stop_cleaning() semantics shift: it becomes "cleaning acknowledged, robot ready for shutdown" — not "return to Idle." State stays Cleaning until Off.

This also clarifies that CleaningHal::start_cleaning() must be gated by admin auth at the handler level, and must flush the job queue as a side effect before transitioning state.

# Mid-pour glass removal: recoverable Error, not WaitingForGlass
Working ──(sensor: glass gone)──► Error
                                  code: "GLASS_REMOVED"
                                  recoverable: true
                                  recovery: PutGlassBack
                                       │
                         glass returns │          timeout
                                       ▼              │
                                    Working      Cancelled ──► Idle
                                    (resume)

This keeps WaitingForGlass clean as a pre-job state. Mid-pour disruption is abnormal → Error. The distinction matters for clients: WaitingForGlass is expected flow, Error is "something went wrong."

RecoveryAction now covers three distinct physical situations:
pub enum RecoveryAction {
    PutGlassBack,    // glass removed mid-pour  → wait → resume or cancel on timeout
    RemoveGlass,     // drink abandoned timeout  → wait → Idle (no further timeout)
    CallResetErrors, // firmware/HAL error       → API call recovers
    None,            // unrecoverable
}

Note the asymmetry in the last two: PutGlassBack has a timeout (→ Cancelled), RemoveGlass waits indefinitely. Once you're in DRINK_ABANDONED error, the robot is physically occupied — no timer needed, just wait for the human.

# Complete state machine
                        ┌─────────────────────────────────────┐
                        │                                     │
  Off ──(on)──► SelfTest ──(fail)──► Error ──(reset)──► SelfTest
                    │
                   (ok)
                    │
          ┌─────────┴──────────┐
          ▼                    ▼
       Idle             Provisioning
          │    ◄───────────────┤  (configured)
          │                    │
          └────────────────────┘
          │  (admin: start_cleaning)
          ▼
       Cleaning ──────────────────────────────── (power off) ──► Off
       [queue cleared]

          │ POST /jobs (from Idle)
          ▼
  WaitingForGlass ──(timeout)──► Cancelled ──► Idle
  reason: NoGlass
  timeout_remaining: u32
          │
          │ glass detected
          ├─ typing=false ────────────────────────────────────────────► Working
          ├─ typing=true, OK ─────────────────────────────────────────► Working
          ├─ typing=true, TOO LARGE ──(warn via SSE)──────────────────► Working
          └─ typing=true, TOO SMALL
                    │
                    ▼
          WaitingForGlass ──(timeout)──► Cancelled ──► Idle
          reason: TooSmall { detected_vol, required_vol }
          timeout_remaining: u32
                    │ glass replaced, OK
                    └──────────────────────────────────────────────────► Working
                                                                            │
                                                              glass removed │
                                                                            ▼
                                                                          Error
                                                                  code: GLASS_REMOVED
                                                                  recovery: PutGlassBack
                                                                  timeout_remaining: u32
                                                                     │          │
                                                             glass   │          │ timeout
                                                             returns │          ▼
                                                                     │      Cancelled ──► Idle
                                                                     ▼
                                                                  Working
                                                                  (resume)
                                                                     │
                                                                  (done)
                                                                     ▼
                                                                 DrinkReady
                                                                 timeout_remaining: u32
                                                                     │
                                                         glass  ─────┘ removed → Idle
                                                         removed
                                                                     │ timeout
                                                                     ▼
                                                                   Error
                                                              code: DRINK_ABANDONED
                                                              recovery: RemoveGlass
                                                              (no timeout — wait indefinitely)
                                                                     │
                                                              glass removed
                                                                     ▼
                                                                    Idle

# Final RobotState type
pub enum RobotState {
    Off,
    SelfTest,
    Provisioning,
    Idle,
    WaitingForGlass {
        job_id: String,
        reason: GlassWaitReason,
        timeout_remaining_secs: Option<u32>,
    },
    Working {
        job_id: String,
        progress_pct: u8,
    },
    DrinkReady {
        job_id: String,
        timeout_remaining_secs: Option<u32>,
    },
    Error {
        code: String,
        message: String,
        job_id: Option<String>,
        recoverable: bool,
        recovery: RecoveryAction,
        timeout_remaining_secs: Option<u32>,  // only for PutGlassBack
    },
    Cleaning,
}

pub enum GlassWaitReason {
    NoGlass,
    TooSmall { detected_volume: f32, required_volume: f32 },
}

pub enum RecoveryAction {
    PutGlassBack,
    RemoveGlass,
    CallResetErrors,
    None,
}
Prepared is gone.

# Config additions
pub struct AdminConfig {
    // ...existing...

    /// 0 = wait indefinitely
    #[serde(default = "default_glass_wait_timeout")]
    pub glass_wait_timeout_secs: u32,      // applies to: WaitingForGlass, GLASS_REMOVED

    /// 0 = wait indefinitely  
    #[serde(default = "default_drink_ready_timeout")]
    pub drink_ready_timeout_secs: u32,     // applies to: DrinkReady only
}

glass_wait_timeout_secs reuses for mid-pour recovery — both are "place the glass correctly" situations, same user intent.

# Capabilities
pub struct Capabilities {
    pub version: String,
    pub level_reporting: LevelReporting,
    pub glass_typing: bool,
    pub simultaneous_channels: u8,
    pub max_queue_depth: u8,
    pub has_cancel_button: bool,
    pub has_power_button: bool,
}
