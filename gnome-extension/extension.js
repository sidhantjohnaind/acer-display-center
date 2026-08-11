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
            if (parsed && parsed.current_values && parsed.current_values.brightness) {
                return parsed.current_values.brightness.current;
            }
        }
    } catch (e) {
        console.error(`AcerMonitor QuickSettings error: ${e}`);
    }
    return 80;
}

const AcerMonitorSlider = GObject.registerClass(
class AcerMonitorSlider extends QuickSettings.QuickSlider {
    _init() {
        super._init({
            iconName: 'display-symbolic',
            title: 'Monitor Brightness',
        });

        let initialVal = getInitialState();
        this.slider.value = initialVal / 100.0;

        let brightTimeout = 0;
        this.slider.connect('notify::value', () => {
            let val = Math.round(this.slider.value * 100);
            if (brightTimeout) {
                GLib.source_remove(brightTimeout);
            }
            brightTimeout = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 100, () => {
                execCli(`brightness ${val}`);
                brightTimeout = 0;
                return GLib.SOURCE_REMOVE;
            });
        });
    }
});

export default class AcerMonitorExtension extends Extension {
    enable() {
        this._slider = new AcerMonitorSlider();
        Main.panel.statusArea.quickSettings.addExternalIndicator(this._slider);
    }

    disable() {
        if (this._slider) {
            this._slider.destroy();
            this._slider = null;
        }
    }
}
