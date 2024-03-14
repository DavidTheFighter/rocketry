use shared::fcu_hal::{FcuAlertCondition, OutputChannel};

use crate::Fcu;


impl<'a> Fcu<'a> {
    pub(crate) fn update_alert_watchdog(&mut self) {
        self.alert_manager.assign_condition(
            FcuAlertCondition::DebugModeEnabled,
            self.debug_info_enabled,
        );

        self.alert_manager.assign_condition(
            FcuAlertCondition::NoIgniterContinuity,
            !self.driver.get_output_channel_continuity(OutputChannel::SolidMotorIgniter),
        );
    }
}
