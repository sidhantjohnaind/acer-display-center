const { Clutter, GLib, St, GObject } = imports.gi;
const Main = imports.ui.main;
const PanelMenu = imports.ui.panelMenu;
const PopupMenu = imports.ui.popupMenu;
const Slider = imports.ui.slider;

function execCli(cmd) {
    try {
        GLib.spawn_command_line_async(`/usr/local/bin/acer_monitor_cli send ${cmd}`);
    } catch (e) {
        GLib.spawn_command_line_async(`/usr/local/bin/acer_monitor_cli ${cmd}`);
    }
}

const AcerMonitorIndicator = GObject.registerClass(
class AcerMonitorIndicator extends PanelMenu.Button {
    _init() {
        super._init(0.0, 'Acer Monitor Control', false);

        // Icon in top panel
        let icon = new St.Icon({
            icon_name: 'display-symbolic',
            style_class: 'system-status-icon',
        });
        this.add_child(icon);

        // Scroll listener on panel icon
        this.connect('scroll-event', (actor, event) => {
            let direction = event.get_scroll_direction();
            if (direction === Clutter.ScrollDirection.UP) {
                execCli('brightness +5 --osd');
            } else if (direction === Clutter.ScrollDirection.DOWN) {
                execCli('brightness -5 --osd');
            }
            return Clutter.EVENT_STOP;
        });

        // Popup Title
        let titleItem = new PopupMenu.PopupMenuItem('🖥️ Acer Monitor Control', { reactive: false });
        this.menu.addMenuItem(titleItem);
        this.menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());

        // Brightness Slider
        let brightBox = new St.BoxLayout({ vertical: false, style_class: 'slider-box' });
        let brightLabel = new St.Label({ text: 'Brightness', y_align: Clutter.ActorAlign.CENTER });
        let brightSlider = new Slider.Slider(0.8);
        brightSlider.connect('notify::value', () => {
            let val = Math.round(brightSlider.value * 100);
            execCli(`brightness ${val}`);
        });
        brightBox.add_child(brightLabel);
        brightBox.add_child(brightSlider);
        let brightItem = new PopupMenu.PopupBaseMenuItem({ reactive: false });
        brightItem.add_child(brightBox);
        this.menu.addMenuItem(brightItem);

        // Contrast Slider
        let contrastBox = new St.BoxLayout({ vertical: false, style_class: 'slider-box' });
        let contrastLabel = new St.Label({ text: 'Contrast', y_align: Clutter.ActorAlign.CENTER });
        let contrastSlider = new Slider.Slider(0.5);
        contrastSlider.connect('notify::value', () => {
            let val = Math.round(contrastSlider.value * 100);
            execCli(`contrast ${val}`);
        });
        contrastBox.add_child(contrastLabel);
        contrastBox.add_child(contrastSlider);
        let contrastItem = new PopupMenu.PopupBaseMenuItem({ reactive: false });
        contrastItem.add_child(contrastBox);
        this.menu.addMenuItem(contrastItem);

        // Volume Slider
        let volumeBox = new St.BoxLayout({ vertical: false, style_class: 'slider-box' });
        let volumeLabel = new St.Label({ text: 'Volume', y_align: Clutter.ActorAlign.CENTER });
        let volumeSlider = new Slider.Slider(1.0);
        volumeSlider.connect('notify::value', () => {
            let val = Math.round(volumeSlider.value * 100);
            execCli(`volume ${val}`);
        });
        volumeBox.add_child(volumeLabel);
        volumeBox.add_child(volumeSlider);
        let volumeItem = new PopupMenu.PopupBaseMenuItem({ reactive: false });
        volumeItem.add_child(volumeBox);
        this.menu.addMenuItem(volumeItem);

        this.menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());

        // Hardware Preset Buttons
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

let indicator;

function init() {}

function enable() {
    indicator = new AcerMonitorIndicator();
    Main.panel.addToStatusArea('acer-monitor-control', indicator);
}

function disable() {
    if (indicator) {
        indicator.destroy();
        indicator = null;
    }
}
