// 動態取得 invoke，防止在 Mock 載入前就被靜態截流
const invoke = (...args) => (window.__TAURI__?.core?.invoke || (async () => ({})))(...args);
import { state } from './state.js';
import { dom } from './dom.js';

// 輔助函數：RGB 轉 Hex
function rgbToHexStr(arr) {
    if (!arr || arr.length < 3) return '#1e1e23';
    return '#' + arr.map((x) => x.toString(16).padStart(2, '0')).join('');
}

export function applyColors(style) {
    if (!style) return;
    const root = document.documentElement;
    const isDark = style.theme !== 'light';

    // --- [基礎與強調顏色] ---
    const bg = isDark ? style.dark_bg : style.light_bg;
    const text = isDark ? style.dark_text : style.light_text;
    const accent = isDark ? style.dark_accent : style.light_accent;
    const danger = isDark ? style.dark_danger : style.light_danger;

    if (bg) root.style.setProperty('--bg-color', `rgb(${bg[0]},${bg[1]},${bg[2]})`);
    if (text) root.style.setProperty('--text-color', `rgb(${text[0]},${text[1]},${text[2]})`);
    if (accent) {
        root.style.setProperty('--accent-color', `rgb(${accent[0]},${accent[1]},${accent[2]})`);
        root.style.setProperty('--accent-color-rgb', `${accent[0]}, ${accent[1]}, ${accent[2]}`);
    }
    if (danger) root.style.setProperty('--danger-color', `rgb(${danger[0]},${danger[1]},${danger[2]})`);

    // --- [間距系統] ---
    if (style.space_sm !== undefined) root.style.setProperty('--space-sm', `${style.space_sm}px`);
    if (style.space_md !== undefined) root.style.setProperty('--space-md', `${style.space_md}px`);
    if (style.space_lg !== undefined) root.style.setProperty('--space-lg', `${style.space_lg}px`);

    // --- [細部元件] ---
    const label = isDark ? style.dark_label : style.light_label;
    const textMuted = isDark ? style.dark_text_muted : style.light_text_muted;
    const btnBg = isDark ? style.dark_btn_bg : style.light_btn_bg;
    const btnText = isDark ? style.dark_btn_text : style.light_btn_text;
    const inputBg = isDark ? style.dark_input_bg : style.light_input_bg;
    const listBg = isDark ? style.dark_list_bg : style.light_list_bg;
    const tabActive = isDark ? style.dark_tab_active : style.light_tab_active;
    const tabInactive = isDark ? style.dark_tab_inactive : style.light_tab_inactive;
    const headerBg = isDark ? style.dark_header_bg : style.light_header_bg;
    const borderColor = isDark ? style.dark_border_color : style.light_border_color;
    const hoverBg = isDark ? style.dark_hover_bg : style.light_hover_bg;
    const sliderBg = isDark ? style.dark_slider_bg : style.light_slider_bg;
    const sliderThumb = isDark ? style.dark_slider_thumb : style.light_slider_thumb;
    const switchBg = isDark ? style.dark_switch_bg : style.light_switch_bg;
    const progressBg = isDark ? style.dark_progress_bg : style.light_progress_bg;

    if (label) root.style.setProperty('--label-color', `rgb(${label[0]},${label[1]},${label[2]})`);
    if (textMuted) root.style.setProperty('--text-muted', `rgb(${textMuted[0]},${textMuted[1]},${textMuted[2]})`);
    if (btnBg) root.style.setProperty('--btn-bg', `rgb(${btnBg[0]},${btnBg[1]},${btnBg[2]})`);
    if (btnText) root.style.setProperty('--btn-text', `rgb(${btnText[0]},${btnText[1]},${btnText[2]})`);
    if (inputBg) root.style.setProperty('--input-bg', `rgb(${inputBg[0]},${inputBg[1]},${inputBg[2]})`);
    if (listBg) root.style.setProperty('--list-bg', `rgb(${listBg[0]},${listBg[1]},${listBg[2]})`);
    if (tabActive) root.style.setProperty('--tab-active-bg', `rgb(${tabActive[0]},${tabActive[1]},${tabActive[2]})`);
    if (tabInactive)
        root.style.setProperty('--tab-inactive-bg', `rgb(${tabInactive[0]},${tabInactive[1]},${tabInactive[2]})`);
    if (headerBg) root.style.setProperty('--header-bg', `rgb(${headerBg[0]},${headerBg[1]},${headerBg[2]})`);
    if (borderColor) {
        root.style.setProperty('--border-color', `rgb(${borderColor[0]},${borderColor[1]},${borderColor[2]})`);
        const alpha = style.border_alpha !== undefined ? style.border_alpha : 0.15;
        root.style.setProperty(
            '--border-light',
            `rgba(${borderColor[0]},${borderColor[1]},${borderColor[2]},${alpha})`
        );
    }
    if (hoverBg) root.style.setProperty('--hover-bg', `rgb(${hoverBg[0]},${hoverBg[1]},${hoverBg[2]})`);
    if (sliderBg) root.style.setProperty('--slider-bg', `rgb(${sliderBg[0]},${sliderBg[1]},${sliderBg[2]})`);
    if (sliderThumb)
        root.style.setProperty('--slider-thumb', `rgb(${sliderThumb[0]},${sliderThumb[1]},${sliderThumb[2]})`);
    if (switchBg) root.style.setProperty('--switch-bg', `rgb(${switchBg[0]},${switchBg[1]},${switchBg[2]})`);
    if (progressBg) root.style.setProperty('--progress-bg', `rgb(${progressBg[0]},${progressBg[1]},${progressBg[2]})`);
    if (style.aurora_1)
        root.style.setProperty('--aurora-1', `rgb(${style.aurora_1[0]},${style.aurora_1[1]},${style.aurora_1[2]})`);
    if (style.aurora_2)
        root.style.setProperty('--aurora-2', `rgb(${style.aurora_2[0]},${style.aurora_2[1]},${style.aurora_2[2]})`);
    if (style.aurora_3)
        root.style.setProperty('--aurora-3', `rgb(${style.aurora_3[0]},${style.aurora_3[1]},${style.aurora_3[2]})`);
    if (style.neon_color)
        root.style.setProperty(
            '--neon-color',
            `rgb(${style.neon_color[0]},${style.neon_color[1]},${style.neon_color[2]})`
        );

    // --- [日誌色彩] ---
    const lInfo = isDark ? style.dark_log_info : style.light_log_info;
    const lWarn = isDark ? style.dark_log_warn : style.light_log_warn;
    const lError = isDark ? style.dark_log_error : style.light_log_error;
    const lSuccess = isDark ? style.dark_log_success : style.light_log_success;
    const lDir = isDark ? style.dark_log_dir : style.light_log_dir;
    const lFile = isDark ? style.dark_log_file : style.light_log_file;

    if (lInfo) root.style.setProperty('--log-info-color', `rgb(${lInfo[0]},${lInfo[1]},${lInfo[2]})`);
    if (lWarn) root.style.setProperty('--log-warn-color', `rgb(${lWarn[0]},${lWarn[1]},${lWarn[2]})`);
    if (lError) root.style.setProperty('--log-error-color', `rgb(${lError[0]},${lError[1]},${lError[2]})`);
    if (lSuccess) root.style.setProperty('--log-success-color', `rgb(${lSuccess[0]},${lSuccess[1]},${lSuccess[2]})`);
    if (lDir) root.style.setProperty('--log-dir-color', `rgb(${lDir[0]},${lDir[1]},${lDir[2]})`);
    if (lFile) root.style.setProperty('--log-file-color', `rgb(${lFile[0]},${lFile[1]},${lFile[2]})`);

    // --- [面板與背景透明度] ---
    if (bg) {
        const pAlpha = style.panel_alpha !== undefined ? style.panel_alpha : 0.03;
        const bAlpha = style.backdrop_alpha !== undefined ? style.backdrop_alpha : 0.6;
        root.style.setProperty('--panel-bg', `rgba(${bg[0]},${bg[1]},${bg[2]},${pAlpha})`);
        root.style.setProperty('--backdrop-bg', `rgba(${bg[0]},${bg[1]},${bg[2]},${bAlpha})`);
    }

    // --- [佈局基礎設定] ---
    if (style.font_size) root.style.setProperty('--font-size', `${style.font_size}px`);
    if (style.btn_rounding_enabled !== false) {
        root.style.setProperty('--border-radius', `${style.btn_rounding_value}px`);
    } else {
        root.style.setProperty('--border-radius', '0px');
    }

    // --- [動畫與進度條] ---
    if (style.progress_pulse_enabled && dom.progressBar) {
        dom.progressBar.style.animation = `pulse ${2.0 / Math.max(0.1, style.progress_pulse_speed)}s infinite`;
    } else if (dom.progressBar) {
        dom.progressBar.style.animation = 'none';
    }

    const bars = [dom.progressBar, dom.batchProgressBar];
    bars.forEach((bar) => {
        if (!bar) return;
        bar.classList.remove('style-aurora', 'style-neon');
        if (style.progress_style === 'aurora') bar.classList.add('style-aurora');
        if (style.progress_style === 'neon') bar.classList.add('style-neon');
    });

    // --- [實例覆寫] ---
    if (style.instance_overrides) {
        for (const [id, ov] of Object.entries(style.instance_overrides)) {
            const el = document.getElementById(id);
            if (!el) continue;

            // 優先選取主題感應顏色
            const cBg = isDark ? ov.dark_bg : ov.light_bg;
            const cText = isDark ? ov.dark_text : ov.light_text;

            if (cBg) el.style.backgroundColor = `rgb(${cBg[0]},${cBg[1]},${cBg[2]})`;
            if (cText) el.style.color = `rgb(${cText[0]},${cText[1]},${cText[2]})`;
            if (ov.rounding !== undefined) el.style.borderRadius = `${ov.rounding}px`;
        }
    }
}

export async function updatePaletteValue() {
    if (!dom.paletteTargetType || !dom.paletteTargetItem || !dom.paletteProperty) return;

    const paletteState = await invoke('derive_palette_state_cmd', {
        input: {
            target_type: dom.paletteTargetType.value,
            target_item: dom.paletteTargetItem.value,
            property: dom.paletteProperty.value,
        },
        style: state.currentStyle,
        lang: dom.uiLang?.value,
    });

    if (dom.paletteClearGroup) dom.paletteClearGroup.style.display = paletteState.show_clear_group ? 'flex' : 'none';
    if (dom.palettePropertyGroup)
        dom.palettePropertyGroup.style.display = paletteState.show_property_group ? 'block' : 'none';
    if (dom.paletteColorGroup) dom.paletteColorGroup.style.display = paletteState.show_color_group ? 'block' : 'none';
    if (dom.paletteNumberGroup)
        dom.paletteNumberGroup.style.display = paletteState.show_number_group ? 'block' : 'none';
    if (dom.labelPaletteNumber) dom.labelPaletteNumber.textContent = paletteState.label_palette_number || '';
    if (dom.labelPaletteColor) dom.labelPaletteColor.textContent = paletteState.label_palette_color || '';
    if (dom.paletteNumber) {
        dom.paletteNumber.value = paletteState.number_value ?? 0;
        dom.paletteNumber.step = `${paletteState.number_step ?? 1}`;
    }
    if (dom.paletteColor) dom.paletteColor.value = paletteState.color_value || '#ffffff';

    // 處理特定元件的文字選項隱藏
    const noTextItems = ['progress-bar', 'batch-progress-bar'];
    if (dom.paletteTargetType.value === 'specific') {
        Array.from(dom.paletteProperty.options).forEach((opt) => {
            if (opt.value === 'text') {
                const hidden = noTextItems.includes(dom.paletteTargetItem.value);
                opt.style.display = hidden ? 'none' : 'block';
                opt.disabled = hidden;
                if (hidden && dom.paletteProperty.value === 'text') dom.paletteProperty.value = 'bg';
            }
        });
    }
}

export async function loadStyle() {
    try {
        const style = (await invoke('get_style_config')) || {};
        state.currentStyle = style;
        applyColors(style);
        const cssVars = await invoke('get_gui_css_vars', { config: style });
        if (cssVars && typeof cssVars === 'object') {
            const root = document.documentElement;
            Object.entries(cssVars).forEach(([k, v]) => root.style.setProperty(k, v));
        }

        // --- 同步常規控制項 ---
        const controls = {
            'font-size': style.font_size || 15,
            'btn-rounding-value': style.btn_rounding_value || 4,
            'pulse-speed': style.progress_pulse_speed || 1,
            'progress-style': style.progress_style || 'default',
        };
        for (const [id, val] of Object.entries(controls)) {
            const el = document.getElementById(id);
            if (el) el.value = val;
        }

        const checks = {
            'chk-btn-rounding': style.btn_rounding_enabled,
            'chk-pulse': style.progress_pulse_enabled,
        };
        for (const [id, val] of Object.entries(checks)) {
            const el = document.getElementById(id);
            if (el) el.checked = !!val;
        }

        // --- 同步顏色選擇器 (支援 Legacy IDs) ---
        const colorMaps = {
            'color-bg': style.dark_bg,
            'color-text': style.dark_text,
            'color-accent': style.dark_accent,
            'color-danger': style.dark_danger,
        };
        for (const [id, val] of Object.entries(colorMaps)) {
            const el = document.getElementById(id);
            if (el && val) el.value = rgbToHexStr(val);
        }

        if (typeof updatePaletteValue === 'function') {
            updatePaletteValue();
        }
    } catch (e) {
        console.error('載入樣式配置失敗:', e);
    }
}

export async function saveStyle() {
    try {
        state.currentStyle = await invoke('build_style_from_form_cmd', {
            base: state.currentStyle,
            input: {
                font_size: dom.fontSize ? dom.fontSize.value : '',
                btn_rounding_enabled: dom.chkBtnRounding ? dom.chkBtnRounding.checked : true,
                btn_rounding_value: dom.btnRoundingValue ? dom.btnRoundingValue.value : '',
                progress_pulse_enabled: dom.chkPulse ? dom.chkPulse.checked : true,
                progress_pulse_speed: dom.pulseSpeed ? dom.pulseSpeed.value : '',
                progress_style: dom.progressStyle ? dom.progressStyle.value : 'default',
                color_bg: document.getElementById('color-bg')?.value || null,
                color_text: document.getElementById('color-text')?.value || null,
                color_accent: document.getElementById('color-accent')?.value || null,
                color_danger: document.getElementById('color-danger')?.value || null,
            },
        });

        applyColors(state.currentStyle);
        await invoke('save_style_config', { config: state.currentStyle });
    } catch (e) {
        console.error('儲存樣式配置失敗:', e);
    }
}

export async function restoreDefaultStyle() {
    try {
        const defaultStyle = await invoke('get_default_style_config');

        state.currentStyle = {
            ...defaultStyle,
            show_palette_settings: state.currentStyle.show_palette_settings,
        };

        applyColors(state.currentStyle);
        await invoke('save_style_config', { config: state.currentStyle });

        if (dom.fontSize) dom.fontSize.value = state.currentStyle.font_size;
        if (dom.chkBtnRounding) dom.chkBtnRounding.checked = state.currentStyle.btn_rounding_enabled;
        if (dom.btnRoundingValue) dom.btnRoundingValue.value = state.currentStyle.btn_rounding_value;
        if (dom.chkPulse) dom.chkPulse.checked = state.currentStyle.progress_pulse_enabled;
        if (dom.pulseSpeed) dom.pulseSpeed.value = state.currentStyle.progress_pulse_speed;
        if (dom.progressStyle) dom.progressStyle.value = state.currentStyle.progress_style || 'default';

        if (typeof updatePaletteValue === 'function') {
            updatePaletteValue();
        }

        const msg =
            state.currentLabels && state.currentLabels.status_style_restored
                ? state.currentLabels.status_style_restored
                : '介面樣式已恢復預設';
        console.log(msg);
    } catch (e) {
        console.error('恢復樣式預設失敗:', e);
    }
}
