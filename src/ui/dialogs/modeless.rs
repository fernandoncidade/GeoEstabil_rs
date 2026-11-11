use crate::ui::theme::GeoEstabilTheme;
use crate::utils::icons::carregar_icone_janela;
use eframe::egui;

pub const ROOT_WINDOW_ID: &str = "__geoestabil_root__";
const ZORDER_MEM_KEY: &str = "geoestabil_window_zorder_stack";

/// Gerenciador da pilha virtual de profundidade (Z-Order) das janelas não-modais (`Modeless`).
///
/// A ordem dos elementos representa profundidade de baixo (`[0]`) para o topo (`[len - 1]`).
/// O último item (`top()`) é a janela visualmente à frente de todas as demais.
#[derive(Debug, Clone)]
pub struct WindowZOrderManager {
    stack: Vec<String>,
}

impl Default for WindowZOrderManager {
    fn default() -> Self {
        Self {
            stack: vec![ROOT_WINDOW_ID.to_string()],
        }
    }
}

impl WindowZOrderManager {
    /// Garante a presença da janela na pilha sem alterar quem está no topo caso já esteja aberta.
    pub fn ensure_present(&mut self, id: &str) {
        if !self.stack.iter().any(|w| w == id) {
            self.stack.push(id.to_string());
        }
    }

    /// Eleva uma janela ativa para o topo absoluto da lista Z-Order (`raise`).
    pub fn raise(&mut self, id: &str) {
        self.stack.retain(|w| w != id);
        self.stack.push(id.to_string());
    }

    /// Remove janela fechada da pilha, preservando a janela principal na base caso vazia.
    pub fn close(&mut self, id: &str) {
        if id == ROOT_WINDOW_ID {
            return;
        }
        self.stack.retain(|w| w != id);
        if self.stack.is_empty() {
            self.stack.push(ROOT_WINDOW_ID.to_string());
        }
    }

    /// Identificador da janela atualmente no topo visual (última da pilha).
    pub fn top(&self) -> Option<&str> {
        self.stack.last().map(|s| s.as_str())
    }

    /// Índice de profundidade (0 = fundo, `len - 1` = topo).
    pub fn depth_index(&self, id: &str) -> Option<usize> {
        self.stack.iter().position(|w| w == id)
    }

    /// Lista completa de janelas abertas em ordem Z (da mais recuada à mais à frente).
    pub fn stack(&self) -> &[String] {
        &self.stack
    }

    /// Calcula deslocamento de cascata determinístico apenas entre janelas filhas abertas.
    pub fn child_cascade_slot(&self, id: &str) -> usize {
        self.stack
            .iter()
            .filter(|w| w.as_str() != ROOT_WINDOW_ID)
            .position(|w| w == id)
            .unwrap_or_else(|| self.stack.len().saturating_sub(1))
    }
}

fn zorder_mem_id() -> egui::Id {
    egui::Id::unique(ZORDER_MEM_KEY)
}

fn with_zorder_mut<F: FnOnce(&mut WindowZOrderManager)>(ctx: &egui::Context, f: F) {
    let key = zorder_mem_id();
    ctx.data_mut(|d| {
        let mut mgr = d.get_temp::<WindowZOrderManager>(key).unwrap_or_default();
        f(&mut mgr);
        d.insert_temp(key, mgr);
    });
}

pub fn ctx_zorder_raise(ctx: &egui::Context, id: &str) {
    with_zorder_mut(ctx, |mgr| mgr.raise(id));
}

pub fn ctx_zorder_ensure(ctx: &egui::Context, id: &str) {
    with_zorder_mut(ctx, |mgr| mgr.ensure_present(id));
}

pub fn ctx_zorder_close(ctx: &egui::Context, id: &str) {
    with_zorder_mut(ctx, |mgr| mgr.close(id));
}

pub fn ctx_zorder_snapshot(ctx: &egui::Context) -> WindowZOrderManager {
    ctx.data(|d| {
        d.get_temp::<WindowZOrderManager>(zorder_mem_id())
            .unwrap_or_default()
    })
}

/// Estado de abertura, foco e posicionamento no mesmo monitor da GUI pai de uma janela não-modal (`Modeless`).
#[derive(Debug, Clone, Default)]
pub struct ModelessState {
    pub is_open: bool,
    pub bring_to_front: bool,
    pub needs_initial_placement: bool,
}

impl ModelessState {
    pub fn open(&mut self) {
        if !self.is_open {
            self.needs_initial_placement = true;
        }
        self.is_open = true;
        self.bring_to_front = true;
    }

    pub fn close(&mut self) {
        self.is_open = false;
        self.bring_to_front = false;
        self.needs_initial_placement = false;
    }

    pub fn toggle(&mut self) {
        if self.is_open {
            self.close();
        } else {
            self.open();
        }
    }
}

/// Calcula posição `[x, y]` e tamanho `[w, h]` ajustado para abrir a janela filha
/// **no mesmo monitor** em que a GUI pai estiver posicionada, com cascata suave e contenção na área útil.
pub fn compute_child_window_rect(
    parent_rect: egui::Rect,
    monitor_work_area: Option<egui::Rect>,
    requested_size: [f32; 2],
    cascade_slot: usize,
) -> ([f32; 2], [f32; 2]) {
    let bounds = monitor_work_area.unwrap_or_else(|| {
        let fallback_w = (parent_rect.width().max(requested_size[0] + 80.0)).max(1280.0);
        let fallback_h = (parent_rect.height().max(requested_size[1] + 80.0)).max(820.0);
        egui::Rect::from_center_size(parent_rect.center(), egui::vec2(fallback_w, fallback_h))
    });

    let avail_w = (bounds.width() - 24.0).max(320.0);
    let avail_h = (bounds.height() - 42.0).max(240.0);
    let cw = requested_size[0].min(avail_w);
    let ch = requested_size[1].min(avail_h);

    let offset = (cascade_slot % 5) as f32 * 26.0;
    let center = parent_rect.center();
    let raw_x = center.x - cw * 0.5 + offset;
    let raw_y = center.y - ch * 0.5 + offset;

    let min_x = bounds.min.x + 8.0;
    let max_x = (bounds.max.x - cw - 8.0).max(min_x);
    let min_y = bounds.min.y + 30.0;
    let max_y = (bounds.max.y - ch - 8.0).max(min_y);

    (
        [raw_x.clamp(min_x, max_x), raw_y.clamp(min_y, max_y)],
        [cw, ch],
    )
}

#[cfg(target_os = "windows")]
fn query_parent_monitor_work_area(ctx: &egui::Context, parent_rect: egui::Rect) -> Option<egui::Rect> {
    use windows_sys::Win32::Foundation::POINT;
    use windows_sys::Win32::Graphics::Gdi::{
        GetMonitorInfoW, MonitorFromPoint, MONITORINFO, MONITOR_DEFAULTTONEAREST,
    };

    let ppp = ctx.pixels_per_point().max(0.5);
    let pt = POINT {
        x: (parent_rect.center().x * ppp).round() as i32,
        y: (parent_rect.center().y * ppp).round() as i32,
    };

    unsafe {
        let hmon = MonitorFromPoint(pt, MONITOR_DEFAULTTONEAREST);
        if hmon.is_null() {
            return None;
        }
        let mut info: MONITORINFO = std::mem::zeroed();
        info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
        if GetMonitorInfoW(hmon, &mut info) != 0 {
            let w = info.rcWork;
            return Some(egui::Rect::from_min_max(
                egui::pos2(w.left as f32 / ppp, w.top as f32 / ppp),
                egui::pos2(w.right as f32 / ppp, w.bottom as f32 / ppp),
            ));
        }
    }
    None
}

#[cfg(not(target_os = "windows"))]
fn query_parent_monitor_work_area(ctx: &egui::Context, parent_rect: egui::Rect) -> Option<egui::Rect> {
    ctx.input(|i| i.viewport().monitor_size).map(|ms| {
        egui::Rect::from_center_size(parent_rect.center(), ms)
    })
}

/// Exibe uma janela não-modal (`ViewportClass::Immediate`) independente no SO com suporte
/// a gerenciamento de profundidade (Z-Order click-to-raise), abertura no mesmo monitor da GUI pai
/// e renderização limpa em testes headless.
#[allow(clippy::too_many_arguments)]
pub fn show_modeless_dialog<F>(
    ctx: &egui::Context,
    id_str: &str,
    title: &str,
    default_size: [f32; 2],
    min_size: [f32; 2],
    dark_mode: bool,
    state: &mut ModelessState,
    mut content_fn: F,
) where
    F: FnMut(&mut egui::Ui, &mut bool),
{
    if !state.is_open {
        ctx_zorder_close(ctx, id_str);
        return;
    }

    ctx_zorder_ensure(ctx, id_str);
    let vp_id = egui::ViewportId::from_hash_of(id_str);

    let (parent_rect_opt, fallback_mon_sz) = ctx.input(|i| {
        (
            i.viewport().outer_rect.or(i.viewport().inner_rect),
            i.viewport().monitor_size,
        )
    });

    let mut builder = egui::ViewportBuilder::default()
        .with_title(title)
        .with_inner_size(default_size)
        .with_min_inner_size(min_size)
        .with_resizable(true)
        .with_decorations(true);

    if let Some(icon) = carregar_icone_janela() {
        builder = builder.with_icon(std::sync::Arc::new(icon));
    }

    let mut initial_pos_to_apply: Option<[f32; 2]> = None;
    if state.needs_initial_placement {
        if let Some(parent_rect) = parent_rect_opt {
            let work_area = query_parent_monitor_work_area(ctx, parent_rect)
                .or_else(|| fallback_mon_sz.map(|ms| egui::Rect::from_center_size(parent_rect.center(), ms)));
            let slot = ctx_zorder_snapshot(ctx).child_cascade_slot(id_str);
            let (pos, clamped_sz) =
                compute_child_window_rect(parent_rect, work_area, default_size, slot);
            builder = builder
                .with_position(egui::pos2(pos[0], pos[1]))
                .with_inner_size(clamped_sz);
            initial_pos_to_apply = Some(pos);
        }
        state.needs_initial_placement = false;
    }

    if state.bring_to_front {
        ctx_zorder_raise(ctx, id_str);
        ctx.send_viewport_cmd_to(vp_id, egui::ViewportCommand::Minimized(false));
        ctx.send_viewport_cmd_to(vp_id, egui::ViewportCommand::Focus);
        if let Some(pos) = initial_pos_to_apply {
            ctx.send_viewport_cmd_to(
                vp_id,
                egui::ViewportCommand::OuterPosition(egui::pos2(pos[0], pos[1])),
            );
        }
        state.bring_to_front = false;
    }

    let mut close_clicked = false;
    let mut close_requested = false;
    let mut raised_by_interaction = false;

    ctx.show_viewport_immediate(vp_id, builder, |ui, _class| {
        GeoEstabilTheme::apply(ui.ctx(), dark_mode);
        *ui.style_mut() = (*ui.ctx().global_style()).clone();

        let (vp_close, esc_pressed, any_click, gained_focus) = ui.ctx().input(|i| {
            let own_vp = i.raw.viewport_id == vp_id;
            let gained = own_vp
                && i.events
                    .iter()
                    .any(|e| matches!(e, egui::Event::WindowFocused(true)));
            (
                i.viewport().close_requested(),
                own_vp && i.key_pressed(egui::Key::Escape),
                own_vp && i.pointer.any_pressed(),
                gained,
            )
        });
        if vp_close {
            close_requested = true;
        }
        if esc_pressed {
            close_clicked = true;
        }
        if any_click || gained_focus {
            raised_by_interaction = true;
            if any_click {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Focus);
            }
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(ui.style()).inner_margin(egui::Margin::same(10)))
            .show(ui, |ui| {
                content_fn(ui, &mut close_clicked);
            });
    });

    if raised_by_interaction {
        ctx_zorder_raise(ctx, id_str);
    }

    if close_clicked || close_requested {
        ctx_zorder_close(ctx, id_str);
        state.close();
    }
}
