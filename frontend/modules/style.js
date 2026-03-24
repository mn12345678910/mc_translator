// frontend/modules/style.js
import { state } from './state.js';
import { rgbToHex } from './utils.js';

const { invoke } = window.__TAURI__ ? window.__TAURI__.core : { invoke: () => {} };

export async function loadStyle() {
    const colorBg = document.getElementById('color-bg');
    const colorText = document.getElementById('color-text');
    const colorBtnBg = document.getElementById('color-btn-bg');
    const colorBtnText = document.getElementById('color-btn-text');
    const fontSize = document.getElementById('font-size');
    const chkBtnRounding = document.getElementById('chk-btn-rounding');
    const btnRoundingValue = document.getElementById('btn-rounding-value');
    const chkPulse = document.getElementById('chk-pulse');
    const pulseSpeed = document.getElementById('pulse-speed');

    try {
        const style = await invoke('get_style_config');
        state.currentStyle = style;

        const isDark = style.theme !== 'light';
        if (colorBg) colorBg.value = rgbToHex(isDark ? style.dark_bg : style.light_bg);
        if (colorText) colorText.value = rgbToHex(isDark ? style.dark_text : style.light_text);
        if (colorBtnBg) colorBtnBg.value = rgbToHex(isDark ? style.dark_btn_bg : style.light_btn_bg);
        if (colorBtnText) colorBtnText.value = rgbToHex(isDark ? style.dark_btn_text : style.light_btn_text);

        if (style.font_size && fontSize) {
            fontSize.value = style.font_size;
            document.documentElement.style.setProperty('--font-size', style.font_size + 'px');
        }
        if (chkBtnRounding) chkBtnRounding.checked = style.btn_rounding_enabled ?? true;
        if (btnRoundingValue) btnRoundingValue.value = style.btn_rounding_value ?? 4.0;
        if (chkPulse) chkPulse.checked = style.progress_pulse_enabled ?? true;
        if (pulseSpeed) pulseSpeed.value = style.progress_pulse_speed ?? 1.0;

        applyColors(style);
    } catch (e) {
        console.error(e);
    }
}

export async function saveStyle() {
    const chkBtnRounding = document.getElementById('chk-btn-rounding');
    const btnRoundingValue = document.getElementById('btn-rounding-value');
    const chkPulse = document.getElementById('chk-pulse');
    const pulseSpeed = document.getElementById('pulse-speed');
    const fsInput = document.getElementById('font-size');

    if (!state.currentStyle) return;
    if (chkBtnRounding) state.currentStyle.btn_rounding_enabled = chkBtnRounding.checked;
    if (btnRoundingValue) state.currentStyle.btn_rounding_value = parseFloat(btnRoundingValue.value) || 4.0;
    if (chkPulse) state.currentStyle.progress_pulse_enabled = chkPulse.checked;
    if (pulseSpeed) state.currentStyle.progress_pulse_speed = parseFloat(pulseSpeed.value) || 1.0;
    if (fsInput) state.currentStyle.font_size = parseFloat(fsInput.value) || 15;

    await invoke('save_style_config', { config: state.currentStyle });
    applyColors(state.currentStyle);
}

export function applyColors(style) {
    const overrideableIds = [
        'btn-translate',
        'btn-pause',
        'btn-stop',
        'btn-browse-file',
        'btn-browse-dir',
        'btn-browse-output',
        'btn-browse-output-open',
        'user-prompt',
        'system-prompt',
        'input-path',
        'output-dir',
        'dict-dialog',
        'log-output',
        'progress-bar',
        'batch-progress-bar',
    ];
    for (const id of overrideableIds) {
        const targetEl = document.getElementById(id);
        if (targetEl) {
            targetEl.style.backgroundColor = '';
            targetEl.style.color = '';
            targetEl.style.borderRadius = '';
        }
    }

    const progressBar = document.getElementById('progress-bar');
    const isDark = style.theme !== 'light';
    const backgroundColor = isDark ? style.dark_bg : style.light_bg;
    const textColor = isDark ? style.dark_text : style.light_text;
    const buttonBgColor = isDark ? style.dark_btn_bg : style.light_btn_bg;
    const buttonTextColor = isDark ? style.dark_btn_text : style.light_btn_text;
    const inputBgColor = isDark ? style.dark_input_bg : style.light_input_bg;
    const listBgColor = isDark ? style.dark_list_bg : style.light_list_bg;
    const activeTabBg = isDark ? style.dark_tab_active : style.light_tab_active;
    const inactiveTabBg = isDark ? style.dark_tab_inactive : style.light_tab_inactive;
    const labelColor = isDark ? style.dark_label : style.light_label;
    const textMuted = isDark ? style.dark_text_muted : style.light_text_muted;
    
    // [NEW] 擴充細部變數
    const headerBg = isDark ? style.dark_header_bg : style.light_header_bg;
    const borderColor = isDark ? style.dark_border_color : style.light_border_color;
    const hoverBg = isDark ? style.dark_hover_bg : style.light_hover_bg;
    const sliderBg = isDark ? style.dark_slider_bg : style.light_slider_bg;
    const sliderThumb = isDark ? style.dark_slider_thumb : style.light_slider_thumb;
    const switchBg = isDark ? style.dark_switch_bg : style.light_switch_bg;
    const progressBg = isDark ? style.dark_progress_bg : style.light_progress_bg;

    if (backgroundColor) document.documentElement.style.setProperty('--bg-color', `rgb(${backgroundColor[0]},${backgroundColor[1]},${backgroundColor[2]})`);
    if (textColor) document.documentElement.style.setProperty('--text-color', `rgb(${textColor[0]},${textColor[1]},${textColor[2]})`);
    if (buttonBgColor) document.documentElement.style.setProperty('--btn-bg', `rgb(${buttonBgColor[0]},${buttonBgColor[1]},${buttonBgColor[2]})`);
    if (buttonTextColor) document.documentElement.style.setProperty('--btn-text', `rgb(${buttonTextColor[0]},${buttonTextColor[1]},${buttonTextColor[2]})`);
    if (inputBgColor)
        document.documentElement.style.setProperty('--input-bg', `rgb(${inputBgColor[0]},${inputBgColor[1]},${inputBgColor[2]})`);
    if (listBgColor) document.documentElement.style.setProperty('--list-bg', `rgb(${listBgColor[0]},${listBgColor[1]},${listBgColor[2]})`);
    if (activeTabBg)
        document.documentElement.style.setProperty(
            '--tab-active-bg',
            `rgb(${activeTabBg[0]},${activeTabBg[1]},${activeTabBg[2]})`
        );
    if (inactiveTabBg)
        document.documentElement.style.setProperty(
            '--tab-inactive-bg',
            `rgb(${inactiveTabBg[0]},${inactiveTabBg[1]},${inactiveTabBg[2]})`
        );
    if (labelColor)
        document.documentElement.style.setProperty(
            '--label-color',
            `rgb(${labelColor[0]},${labelColor[1]},${labelColor[2]})`
        );
    
    if (textMuted)
        document.documentElement.style.setProperty(
            '--text-muted',
            `rgb(${textMuted[0]},${textMuted[1]},${textMuted[2]})`
        );

    // [NEW] 套用細部變數
    if (headerBg) document.documentElement.style.setProperty('--header-bg', `rgb(${headerBg[0]},${headerBg[1]},${headerBg[2]})`);
    if (borderColor) document.documentElement.style.setProperty('--border-color', `rgb(${borderColor[0]},${borderColor[1]},${borderColor[2]})`);
    if (hoverBg) document.documentElement.style.setProperty('--hover-bg', `rgb(${hoverBg[0]},${hoverBg[1]},${hoverBg[2]})`);
    if (sliderBg) document.documentElement.style.setProperty('--slider-bg', `rgb(${sliderBg[0]},${sliderBg[1]},${sliderBg[2]})`);
    if (sliderThumb) document.documentElement.style.setProperty('--slider-thumb', `rgb(${sliderThumb[0]},${sliderThumb[1]},${sliderThumb[2]})`);
    if (switchBg) document.documentElement.style.setProperty('--switch-bg', `rgb(${switchBg[0]},${switchBg[1]},${switchBg[2]})`);
    if (progressBg) document.documentElement.style.setProperty('--progress-bg', `rgb(${progressBg[0]},${progressBg[1]},${progressBg[2]})`);

    if (style.font_size) document.documentElement.style.setProperty('--font-size', `${style.font_size}px`);

    if (style.btn_rounding_enabled !== false) {
        document.documentElement.style.setProperty('--border-radius', `${style.btn_rounding_value ?? 4.0}px`);
    } else {
        document.documentElement.style.setProperty('--border-radius', '0px');
    }

    if (style.progress_pulse_enabled && progressBar) {
        const speed = style.progress_pulse_speed ?? 1.0;
        progressBar.style.animation = `pulse ${2.0 / Math.max(0.1, speed)}s infinite`;
    } else if (progressBar) {
        progressBar.style.animation = 'none';
    }

    if (style.instance_overrides) {
        for (const [id, override] of Object.entries(style.instance_overrides)) {
            const targetEl = document.getElementById(id);
            if (targetEl) {
                if (override.bg)
                    targetEl.style.backgroundColor = `rgb(${override.bg[0]},${override.bg[1]},${override.bg[2]})`;
                if (override.text) targetEl.style.color = `rgb(${override.text[0]},${override.text[1]},${override.text[2]})`;
                if (override.rounding !== undefined) targetEl.style.borderRadius = `${override.rounding}px`;
            }
        }
    }
}

export function updatePaletteValue() {
    const paletteTargetType = document.getElementById('palette-target-type');
    const paletteTargetItem = document.getElementById('palette-target-item');
    const paletteProperty = document.getElementById('palette-property');
    const paletteClearGroup = document.getElementById('palette-clear-group');
    const palettePropertyGroup = document.getElementById('palette-property-group');
    const paletteColorGroup = document.getElementById('palette-color-group');
    const paletteRoundingGroup = document.getElementById('palette-rounding-group');
    const paletteColor = document.getElementById('palette-color');
    const paletteRounding = document.getElementById('palette-rounding');
    const labelPaletteColor = document.getElementById('label-palette-color');

    if (!paletteTargetType || !paletteTargetItem) return;

    const isSpecific = paletteTargetType.value === 'specific';
    const target = paletteTargetItem.value;

    const noTextItems = ['progress-bar', 'batch-progress-bar'];
    if (isSpecific && paletteProperty) {
        Array.from(paletteProperty.options).forEach((opt) => {
            if (opt.value === 'text') {
                opt.style.display = noTextItems.includes(target) ? 'none' : 'block';
                opt.disabled = noTextItems.includes(target);
                if (noTextItems.includes(target) && paletteProperty.value === 'text') {
                    paletteProperty.value = 'bg';
                }
            }
        });
    }

    const prop = paletteProperty ? paletteProperty.value : 'bg';

    if (paletteClearGroup) paletteClearGroup.style.display = isSpecific ? 'flex' : 'none';

    function rgbToHexStr(arr) {
        if (!arr || arr.length < 3) return '#1e1e23';
        return '#' + arr.map((x) => x.toString(16).padStart(2, '0')).join('');
    }

    if (!isSpecific) {
        if (palettePropertyGroup) palettePropertyGroup.style.display = 'none';
        if (paletteColorGroup) paletteColorGroup.style.display = 'block';
        if (paletteRoundingGroup) paletteRoundingGroup.style.display = 'none';

        const color = state.currentStyle[target];
        if (color && paletteColor) paletteColor.value = rgbToHexStr(color);
    } else {
        if (palettePropertyGroup) palettePropertyGroup.style.display = 'block';
        const override = state.currentStyle.instance_overrides ? state.currentStyle.instance_overrides[target] : null;

        if (prop === 'rounding') {
            if (paletteColorGroup) paletteColorGroup.style.display = 'none';
            if (paletteRoundingGroup) paletteRoundingGroup.style.display = 'block';
            if (paletteRounding)
                paletteRounding.value = override && override.rounding !== undefined ? override.rounding : 4;
        } else {
            if (paletteColorGroup) paletteColorGroup.style.display = 'block';
            if (paletteRoundingGroup) paletteRoundingGroup.style.display = 'none';
            if (labelPaletteColor)
                labelPaletteColor.textContent =
                    prop === 'bg' ? state.currentLabels.label_bg_color : state.currentLabels.label_text_color;

            let color = override ? (prop === 'bg' ? override.bg : override.text) : null;
            if (paletteColor) paletteColor.value = color ? rgbToHexStr(color) : '#ffffff';
        }
    }
}
