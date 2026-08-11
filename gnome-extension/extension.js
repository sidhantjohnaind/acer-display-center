import { Extension } from 'resource:///org/gnome/shell/extensions/extension.js';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import * as PanelMenu from 'resource:///org/gnome/shell/ui/panelMenu.js';
import * as PopupMenu from 'resource:///org/gnome/shell/ui/popupMenu.js';
import * as Slider from 'resource:///org/gnome/shell/ui/slider.js';
import St from 'gi://St';
import GLib from 'gi://GLib';
import Clutter from 'gi://Clutter';
import GObject from 'gi://GObject';

function execCli(cmd) {
    try {
        GLib.spawn_command_line_async(`/usr/local/bin/acer_monitor_cli send ${cmd}`);
    } catch (e) {
        try {
            GLib.spawn_command_line_async(`/usr/local/bin/acer_monitor_cli ${cmd}`);
        } catch (err) {
            console.error(`AcerMonitor error: ${err}`);
        }
    }
}

function getInitialState() {
    try {
        let [res, stdout] = GLib.spawn_command_line_sync('/usr/local/bin/acer_monitor_cli info --json');
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
        console.error(`AcerMonitor state error: ${e}`);
    }
    return { brightness: 80, contrast: 50, volume: 100 };
}

const AcerMonitorIndicator = GObject.registerClass(
class AcerMonitorIndicator extends PanelMenu.Button {
    _init() {
        super._init(0.0, 'Acer Monitor Control', false);

        let state = getInitialState();

        let icon = new St.Icon({
            icon_name: 'display-symbolic',
            style_class: 'system-status-icon',
        });
        this.add_child(icon);

        this.connect('scroll-event', (actor, event) => {
            let direction = event.get_scroll_direction();
            if (direction === Clutter.ScrollDirection.UP) {
                execCli('brightness +5 --osd');
            } else if (direction === Clutter.ScrollDirection.DOWN) {
                execCli('brightness -5 --osd');
            }
            return Clutter.EVENT_STOP;
        });

        let titleItem = new PopupMenu.PopupMenuItem('🖥️ Acer Monitor Control', { reactive: false });
        this.menu.addMenuItem(titleItem);
        this.menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());

        // Brightness Section with 150ms I2C Debounce
        let brightTimeout = 0;
        let brightLabelItem = new PopupMenu.PopupMenuItem(`Brightness (${state.brightness}%)`, { reactive: false });
        this.menu.addMenuItem(brightLabelItem);

        let brightSliderItem = new PopupMenu.PopupBaseMenuItem({ reactive: true, activate: false });
        let brightSlider = new Slider.Slider(state.brightness / 100.0);
        brightSlider.connect('notify::value', () => {
            let val = Math.round(brightSlider.value * 100);
            brightLabelItem.label.set_text(`Brightness (${val}%)`);
            if (brightTimeout) {
                GLib.source_remove(brightTimeout);
                brightTimeout = 0;
            }
            brightTimeout = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 150, () => {
                execCli(`brightness ${val}`);
                brightTimeout = 0;
                return GLib.SOURCE_REMOVE;
            });
        });
        brightSliderItem.add_child(brightSlider);
        this.menu.addMenuItem(brightSliderItem);

        // Contrast Section with 150ms I2C Debounce
        let contrastTimeout = 0;
        let contrastLabelItem = new PopupMenu.PopupMenuItem(`Contrast (${state.contrast}%)`, { reactive: false });
        this.menu.addMenuItem(contrastLabelItem);

        let contrastSliderItem = new PopupMenu.PopupBaseMenuItem({ reactive: true, activate: false });
        let contrastSlider = new Slider.Slider(state.contrast / 100.0);
        contrastSlider.connect('notify::value', () => {
            let val = Math.round(contrastSlider.value * 100);
            contrastLabelItem.label.set_text(`Contrast (${val}%)`);
            if (contrastTimeout) {
                GLib.source_remove(contrastTimeout);
                contrastTimeout = 0;
            }
            contrastTimeout = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 150, () => {
                execCli(`contrast ${val}`);
                contrastTimeout = 0;
                return GLib.SOURCE_REMOVE;
            });
        });
        contrastSliderItem.add_child(contrastSlider);
        this.menu.addMenuItem(contrastSliderItem);

        // Volume Section with 150ms I2C Debounce
        let volumeTimeout = 0;
        let volumeLabelItem = new PopupMenu.PopupMenuItem(`Volume (${state.volume}%)`, { reactive: false });
        this.menu.addMenuItem(volumeLabelItem);

        let volumeSliderItem = new PopupMenu.PopupBaseMenuItem({ reactive: true, activate: false });
        let volumeSlider = new Slider.Slider(state.volume / 100.0);
        volumeSlider.connect('notify::value', () => {
            let val = Math.round(volumeSlider.value * 100);
            volumeLabelItem.label.set_text(`Volume (${val}%)`);
            if (volumeTimeout) {
                GLib.source_remove(volumeTimeout);
                volumeTimeout = 0;
            }
            volumeTimeout = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 150, () => {
                execCli(`volume ${val}`);
                volumeTimeout = 0;
                return GLib.SOURCE_REMOVE;
            });
        });
        volumeSliderItem.add_child(volumeSlider);
        this.menu.addMenuItem(volumeSliderItem);

        this.menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());

        // Presets Header
        let presetHeader = new PopupMenu.PopupMenuItem('Mode Presets', { reactive: false });
        this.menu.addMenuItem(presetHeader);

        let itemStandard = new PopupMenu.PopupMenuItem('  Standard Mode');
        itemStandard.connect('activate', () => execCli('preset standard'));
        this.menu.addMenuItem(itemStandard);

        let itemEco = new PopupMenu.PopupMenuItem('  ECO Power Saver');
        itemEco.connect('activate', () => execCli('preset eco'));
        this.menu.addMenuItem(itemEco);

        let itemHdr = new PopupMenu.PopupMenuItem('  HDR Gaming Mode');
        itemHdr.connect('activate', () => execCli('preset hdr'));
        this.menu.addMenuItem(itemHdr);

        let itemReading = new PopupMenu.PopupMenuItem('  Reading Mode (Warm)');
        itemReading.connect('activate', () => execCli('colortemp warm'));
        this.menu.addMenuItem(itemReading);
    }
});

export default class AcerMonitorExtension extends Extension {
    enable() {
        this._indicator = new AcerMonitorIndicator();
        Main.panel.addToStatusArea(this.uuid, this._indicator);
    }

    disable() {
        if (this._indicator) {
            this._indicator.destroy();
            this._indicator = null;
        }
    }
}
