import { Extension } from 'resource:///org/gnome/shell/extensions/extension.js';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import * as QuickSettings from 'resource:///org/gnome/shell/ui/quickSettings.js';
import GLib from 'gi://GLib';
import GObject from 'gi://GObject';

function execCli(cmd) {
    GLib.spawn_command_line_async(`/usr/local/bin/amctl ${cmd}`);
}

function getInitialState() {
    try {
        let [res, stdout] = GLib.spawn_command_line_sync('/usr/local/bin/amctl info --json');
        if (res && stdout) {
            let str = new TextDecoder().decode(stdout);
            let parsed = JSON.parse(str);
            if (parsed && parsed.current_values) {
                return {
                    brightness: parsed.current_values.brightness ? parsed.current_values.brightness.current : 80,
                    contrast: parsed.current_values.contrast ? parsed.current_values.contrast.current : 50,
                    volume: parsed.current_values.volume ? parsed.current_values.volume.current : 100
                };
            }
        }
    } catch (e) {
        console.error(`AcerMonitor QuickSettings error: ${e}`);
    }
    return { brightness: 80, contrast: 50, volume: 100 };
}

// Brightness Slider in QuickSettings
const AcerBrightnessSlider = GObject.registerClass(
class AcerBrightnessSlider extends QuickSettings.QuickSlider {
    _init(initialVal) {
        super._init({
            iconName: 'display-brightness-symbolic',
        });
        this.slider.value = initialVal / 100.0;

        let timeout = 0;
        this.slider.connect('notify::value', () => {
            let val = Math.round(this.slider.value * 100);
            if (timeout) GLib.source_remove(timeout);
            timeout = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 100, () => {
                execCli(`brightness ${val}`);
                timeout = 0;
                return GLib.SOURCE_REMOVE;
            });
        });
    }
});

// Contrast Slider in QuickSettings
const AcerContrastSlider = GObject.registerClass(
class AcerContrastSlider extends QuickSettings.QuickSlider {
    _init(initialVal) {
        super._init({
            iconName: 'display-symbolic',
        });
        this.slider.value = initialVal / 100.0;

        let timeout = 0;
        this.slider.connect('notify::value', () => {
            let val = Math.round(this.slider.value * 100);
            if (timeout) GLib.source_remove(timeout);
            timeout = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 100, () => {
                execCli(`contrast ${val}`);
                timeout = 0;
                return GLib.SOURCE_REMOVE;
            });
        });
    }
});

// Volume Slider in QuickSettings
const AcerVolumeSlider = GObject.registerClass(
class AcerVolumeSlider extends QuickSettings.QuickSlider {
    _init(initialVal) {
        super._init({
            iconName: 'audio-speakers-symbolic',
        });
        this.slider.value = initialVal / 100.0;

        let timeout = 0;
        this.slider.connect('notify::value', () => {
            let val = Math.round(this.slider.value * 100);
            if (timeout) GLib.source_remove(timeout);
            timeout = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 100, () => {
                execCli(`volume ${val}`);
                timeout = 0;
                return GLib.SOURCE_REMOVE;
            });
        });
    }
});

// Presets Menu Toggle in QuickSettings
const AcerPresetsToggle = GObject.registerClass(
class AcerPresetsToggle extends QuickSettings.QuickMenuToggle {
    _init() {
        super._init({
            title: 'Presets',
            iconName: 'video-display-symbolic',
            toggleMode: false,
        });

        this.menu.setHeader('video-display-symbolic', 'Presets');

        let modes = [
            { name: 'User Mode', cmd: 'preset user' },
            { name: 'Standard Mode', cmd: 'preset standard' },
            { name: 'ECO Power Saver', cmd: 'preset eco' },
            { name: 'Graphics Mode', cmd: 'preset graphics' },
            { name: 'HDR Mode', cmd: 'preset hdr' },
            { name: 'Action Gaming', cmd: 'preset action' },
            { name: 'Racing Mode', cmd: 'preset racing' },
            { name: 'Sports Mode', cmd: 'preset sports' },
        ];

        for (let m of modes) {
            this.menu.addAction(m.name, () => execCli(m.cmd));
        }
    }
});

export default class AcerMonitorExtension extends Extension {
    enable() {
        let state = getInitialState();

        this._brightSlider = new AcerBrightnessSlider(state.brightness);
        this._contrastSlider = new AcerContrastSlider(state.contrast);
        this._volumeSlider = new AcerVolumeSlider(state.volume);
        this._presetsToggle = new AcerPresetsToggle();

        Main.panel.statusArea.quickSettings.addExternalIndicator(this._brightSlider);
        Main.panel.statusArea.quickSettings.addExternalIndicator(this._contrastSlider);
        Main.panel.statusArea.quickSettings.addExternalIndicator(this._volumeSlider);
        Main.panel.statusArea.quickSettings.addItem(this._presetsToggle);
    }

    disable() {
        if (this._brightSlider) { this._brightSlider.destroy(); this._brightSlider = null; }
        if (this._contrastSlider) { this._contrastSlider.destroy(); this._contrastSlider = null; }
        if (this._volumeSlider) { this._volumeSlider.destroy(); this._volumeSlider = null; }
        if (this._presetsToggle) { this._presetsToggle.destroy(); this._presetsToggle = null; }
    }
}
