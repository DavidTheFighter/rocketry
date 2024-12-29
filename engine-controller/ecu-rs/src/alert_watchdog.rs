use shared::ecu_hal::EcuAlert;

use crate::Ecu;

impl<'a> Ecu<'a> {
    pub(crate) fn update_alert_watchdog(&mut self) {
        self.alert_manager
            .assign_condition(EcuAlert::DebugModeEnabled, self.debug_info_enabled);
    }
}
