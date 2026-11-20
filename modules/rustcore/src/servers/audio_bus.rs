//! `servers/audio/audio_bus_layout.*`: bus state.
//!
//! A layout starts with a `Master` bus that cannot be removed. Each bus has a
//! volume in dB, mute, solo, FX bypass and an optional send target. `solo`
//! follows the engine's rule: when any bus is soloed, only soloed (and unmuted)
//! buses are active; otherwise every unmuted bus is.
//!
//! Not ported: the DSP graph, effects and `AudioServer` mixing.

#[derive(Clone, Debug)]
pub struct AudioBus {
    pub name: String,
    pub volume_db: f32,
    pub mute: bool,
    pub solo: bool,
    pub bypass_fx: bool,
    /// Index of the destination bus, `None` for the master.
    pub send: Option<usize>,
}

#[derive(Clone, Debug)]
pub struct AudioBusLayout {
    buses: Vec<AudioBus>,
}

impl Default for AudioBusLayout {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioBusLayout {
    pub fn new() -> Self {
        Self {
            buses: vec![AudioBus {
                name: "Master".to_string(),
                volume_db: 0.0,
                mute: false,
                solo: false,
                bypass_fx: false,
                send: None,
            }],
        }
    }

    pub fn get_bus_count(&self) -> usize {
        self.buses.len()
    }

    pub fn get_bus(&self, index: usize) -> Option<&AudioBus> {
        self.buses.get(index)
    }

    pub fn get_bus_name(&self, index: usize) -> Option<&str> {
        self.buses.get(index).map(|b| b.name.as_str())
    }

    pub fn index_of(&self, name: &str) -> Option<usize> {
        self.buses.iter().position(|b| b.name == name)
    }

    pub fn add_bus(&mut self, name: &str) -> usize {
        self.buses.push(AudioBus {
            name: name.to_string(),
            volume_db: 0.0,
            mute: false,
            solo: false,
            bypass_fx: false,
            send: None,
        });
        self.buses.len() - 1
    }

    /// `remove_bus`: the master bus (index 0) cannot be removed.
    pub fn remove_bus(&mut self, index: usize) -> bool {
        if index == 0 || index >= self.buses.len() {
            return false;
        }
        self.buses.remove(index);
        // Sends pointing past the removed bus would dangle; drop them.
        for (i, bus) in self.buses.iter_mut().enumerate() {
            match bus.send {
                Some(target) if target == index => bus.send = None,
                Some(target) if target > index => bus.send = Some(target - 1),
                _ => {}
            }
            let _ = i;
        }
        true
    }

    pub fn set_bus_volume_db(&mut self, index: usize, volume_db: f32) -> bool {
        match self.buses.get_mut(index) {
            Some(b) => {
                b.volume_db = volume_db;
                true
            }
            None => false,
        }
    }

    pub fn get_bus_volume_db(&self, index: usize) -> Option<f32> {
        self.buses.get(index).map(|b| b.volume_db)
    }

    pub fn set_bus_mute(&mut self, index: usize, mute: bool) -> bool {
        match self.buses.get_mut(index) {
            Some(b) => {
                b.mute = mute;
                true
            }
            None => false,
        }
    }

    pub fn is_bus_mute(&self, index: usize) -> bool {
        self.buses.get(index).map(|b| b.mute).unwrap_or(false)
    }

    pub fn set_bus_solo(&mut self, index: usize, solo: bool) -> bool {
        match self.buses.get_mut(index) {
            Some(b) => {
                b.solo = solo;
                true
            }
            None => false,
        }
    }

    pub fn is_bus_solo(&self, index: usize) -> bool {
        self.buses.get(index).map(|b| b.solo).unwrap_or(false)
    }

    pub fn set_bus_bypass_fx(&mut self, index: usize, bypass: bool) -> bool {
        match self.buses.get_mut(index) {
            Some(b) => {
                b.bypass_fx = bypass;
                true
            }
            None => false,
        }
    }

    pub fn is_bus_bypassing_fx(&self, index: usize) -> bool {
        self.buses.get(index).map(|b| b.bypass_fx).unwrap_or(false)
    }

    /// Send target must exist and cannot be the bus itself.
    pub fn set_bus_send(&mut self, index: usize, send: Option<usize>) -> bool {
        if index >= self.buses.len() {
            return false;
        }
        if let Some(target) = send {
            if target >= self.buses.len() || target == index {
                return false;
            }
        }
        self.buses[index].send = send;
        true
    }

    pub fn get_bus_send(&self, index: usize) -> Option<usize> {
        self.buses.get(index).and_then(|b| b.send)
    }

    /// Whether the bus contributes audio, given mute/solo.
    pub fn is_bus_active(&self, index: usize) -> bool {
        let Some(bus) = self.buses.get(index) else {
            return false;
        };
        if bus.mute {
            return false;
        }
        if self.buses.iter().any(|b| b.solo) {
            bus.solo
        } else {
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn master_bus_is_created_and_protected() {
        let mut l = AudioBusLayout::new();
        assert_eq!(l.get_bus_count(), 1);
        assert_eq!(l.get_bus_name(0), Some("Master"));
        assert_eq!(l.get_bus_volume_db(0), Some(0.0));
        assert!(!l.remove_bus(0));
        assert_eq!(l.get_bus_count(), 1);
    }

    #[test]
    fn add_and_remove_buses() {
        let mut l = AudioBusLayout::new();
        let music = l.add_bus("Music");
        let sfx = l.add_bus("SFX");
        assert_eq!((music, sfx), (1, 2));
        assert_eq!(l.index_of("SFX"), Some(2));
        assert_eq!(l.index_of("Voice"), None);

        assert!(l.remove_bus(1));
        assert_eq!(l.get_bus_count(), 2);
        // SFX shifted down to index 1.
        assert_eq!(l.get_bus_name(1), Some("SFX"));
        assert!(!l.remove_bus(9));
    }

    #[test]
    fn volume_mute_solo_and_bypass() {
        let mut l = AudioBusLayout::new();
        let a = l.add_bus("A");
        let b = l.add_bus("B");

        assert!(l.set_bus_volume_db(a, -6.0));
        assert_eq!(l.get_bus_volume_db(a), Some(-6.0));
        assert!(!l.set_bus_volume_db(9, 0.0));

        assert!(l.set_bus_mute(b, true));
        assert!(l.is_bus_mute(b));

        // No solo: both unmuted buses are active.
        assert!(l.is_bus_active(a));
        assert!(!l.is_bus_active(b));

        // Solo A: only A is active, even if B were unmuted.
        assert!(l.set_bus_solo(a, true));
        l.set_bus_mute(b, false);
        assert!(l.is_bus_active(a));
        assert!(!l.is_bus_active(b));
        assert!(l.is_bus_active(0) == false, "master is not soloed");

        assert!(l.set_bus_bypass_fx(a, true));
        assert!(l.is_bus_bypassing_fx(a));
    }

    #[test]
    fn sends_must_target_another_existing_bus() {
        let mut l = AudioBusLayout::new();
        let a = l.add_bus("A");
        let b = l.add_bus("B");

        assert!(l.set_bus_send(a, Some(0)));
        assert_eq!(l.get_bus_send(a), Some(0));
        // Self-send and out-of-range targets are rejected.
        assert!(!l.set_bus_send(a, Some(a)));
        assert!(!l.set_bus_send(a, Some(99)));
        assert_eq!(l.get_bus_send(a), Some(0));

        // Removing a bus drops sends that pointed at it.
        l.set_bus_send(a, Some(b));
        assert!(l.remove_bus(b));
        assert_eq!(l.get_bus_send(a), None);
    }
}
