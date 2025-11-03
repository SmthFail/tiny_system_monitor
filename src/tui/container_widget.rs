// src/tui/container_widget.rs

use super::{
    widget::{Widget, Rect, ChildConstraints, WidgetBox},
    buffer::{Buffer, Cell},
    app_error::AppError,
};

pub enum Layout {
    Vertical,
    Horizontal,
    Grid,
}

#[derive(Copy, Clone)]
pub enum Alignment {
    Start,
    Center,
    End,
}

pub struct Container {
    pub width: Option<u16>,   // трактуем как максимум (cap), если Some
    pub height: Option<u16>,  // трактуем как максимум (cap), если Some
    pub children: Vec<Box<dyn Widget>>,
    pub layout: Layout,
    pub alignment: Alignment,
}

impl Container {
    pub fn new(
        width: Option<u16>,
        height: Option<u16>,
        layout: Layout,
        alignment: Alignment,
    ) -> Self {
        Self {
            children: Vec::new(),
            width,
            height,
            layout,
            alignment,
        }
    }

    pub fn add_child<I: Into<WidgetBox>>(&mut self, w: I) {
        self.children.push(w.into().0);
    }

    /// статический хелпер, чтобы не брать &self в циклах iter_mut
    fn draw_overflow_text(alignment: Alignment, buff: &mut Buffer, rect: Rect, label: &str) {
        if rect.width == 0 || rect.height == 0 {
            return;
        }
        let len = label.chars().count() as u16;
        let start_x = match alignment {
            Alignment::Center => {
                if len < rect.width { rect.x + (rect.width - len) / 2 } else { rect.x }
            }
            Alignment::End => {
                if len < rect.width { rect.x + rect.width - len } else { rect.x }
            }
            Alignment::Start => rect.x,
        };
        let mut x = start_x;
        for ch in label.chars().take(rect.width as usize) {
            buff.set_cell(x, rect.y, Cell::new(ch));
            x = x.saturating_add(1);
            if x >= rect.x + rect.width { break; }
        }
    }

    fn make_columns(heights: &[u16], max_height: u16) -> Vec<Vec<usize>> {
        let mut columns: Vec<Vec<usize>> = vec![Vec::new()];
        let mut current_height: u16 = 0;

        for (idx, &h) in heights.iter().enumerate() {
            if !columns.last().unwrap().is_empty() && current_height.saturating_add(h) > max_height {
                columns.push(Vec::new());
                current_height = 0;
            }
            columns.last_mut().unwrap().push(idx);
            current_height = current_height.saturating_add(h);
        }
        columns
    }

    /// вычисление размеров детей с учётом min/max:
    /// - начинаем с min_* каждого
    /// - остаток по оси layout раздаём равномерно, до max_* (или бесконечности, если None)
    fn calculate_childs_size(&self, area: Rect) -> Option<Vec<(u16, u16)>> {
        if self.children.is_empty() { return None; }

        #[derive(Clone, Copy)]
        struct Need { min_w: u16, max_w: Option<u16>, min_h: u16, max_h: Option<u16> }

        let needs: Vec<Need> = self.children.iter().map(|ch| {
            let c = ch.get_constraints();
            Need {
                min_w: c.min_width.unwrap_or(1),
                max_w: c.max_width,
                min_h: c.min_height.unwrap_or(1),
                max_h: c.max_height,
            }
        }).collect();

        let mut sizes: Vec<(u16,u16)> = vec![(1,1); self.children.len()];

        match self.layout {
            Layout::Vertical => {
                // всем ширина = clamp(area.width, [min_w..max_w])
                for (i, n) in needs.iter().enumerate() {
                    let w = area.width.min(n.max_w.unwrap_or(area.width)).max(n.min_w);
                    sizes[i].0 = w;
                }
                // высоты: сначала минимумы
                let mut base: u16 = 0;
                for (i, n) in needs.iter().enumerate() {
                    let h0 = n.min_h.min(area.height);
                    sizes[i].1 = h0;
                    base = base.saturating_add(h0);
                }
                // раздать остаток
                let mut rem = area.height.saturating_sub(base);
                if rem > 0 {
                    let mut flex: Vec<usize> = (0..needs.len()).filter(|&i| {
                        let cap = needs[i].max_h.map(|mh| mh.saturating_sub(sizes[i].1)).unwrap_or(u16::MAX);
                        cap > 0
                    }).collect();
                    'outer: while rem > 0 && !flex.is_empty() {
                        for &i in &flex {
                            let cap = needs[i].max_h.map(|mh| mh.saturating_sub(sizes[i].1)).unwrap_or(u16::MAX);
                            if cap > 0 {
                                sizes[i].1 = sizes[i].1.saturating_add(1);
                                rem -= 1;
                                if rem == 0 { break 'outer; }
                            }
                        }
                        flex.retain(|&i| needs[i].max_h.map(|mh| sizes[i].1 < mh).unwrap_or(true));
                    }
                }
            }
            Layout::Horizontal => {
                // всем высота = clamp(area.height, [min_h..max_h])
                for (i, n) in needs.iter().enumerate() {
                    let h = area.height.min(n.max_h.unwrap_or(area.height)).max(n.min_h);
                    sizes[i].1 = h;
                }
                // ширины: сначала минимумы
                let mut base: u16 = 0;
                for (i, n) in needs.iter().enumerate() {
                    let w0 = n.min_w.min(area.width);
                    sizes[i].0 = w0;
                    base = base.saturating_add(w0);
                }
                // раздать остаток
                let mut rem = area.width.saturating_sub(base);
                if rem > 0 {
                    let mut flex: Vec<usize> = (0..needs.len()).filter(|&i| {
                        let cap = needs[i].max_w.map(|mw| mw.saturating_sub(sizes[i].0)).unwrap_or(u16::MAX);
                        cap > 0
                    }).collect();
                    'outer: while rem > 0 && !flex.is_empty() {
                        for &i in &flex {
                            let cap = needs[i].max_w.map(|mw| mw.saturating_sub(sizes[i].0)).unwrap_or(u16::MAX);
                            if cap > 0 {
                                sizes[i].0 = sizes[i].0.saturating_add(1);
                                rem -= 1;
                                if rem == 0 { break 'outer; }
                            }
                        }
                        flex.retain(|&i| needs[i].max_w.map(|mw| sizes[i].0 < mw).unwrap_or(true));
                    }
                }
            }
            Layout::Grid => {
                // для грида считаем высоты «плиток» = clamp(min_h..max_h..area.height)
                for (i, n) in needs.iter().enumerate() {
                    let h = area.height.min(n.max_h.unwrap_or(area.height)).max(n.min_h);
                    sizes[i].1 = h.max(1);
                    sizes[i].0 = 0; // посчитается позже колонками
                }
            }
        }

        Some(sizes)
    }

    // ---------- drawing ----------

    fn _draw_vertical(&mut self, capped: Rect, buff: &mut Buffer, sizes: Vec<(u16,u16)>) -> Result<(), AppError> {
        let align = self.alignment;
        let mut y = capped.y;

        for (i, child) in self.children.iter_mut().enumerate() {
            if y >= capped.y + capped.height { break; }

            let desired_h = sizes[i].1.max(1);
            let avail = (capped.y + capped.height).saturating_sub(y);
            let h = desired_h.min(avail);

            if h == 0 { break; }

            let rect = Rect { x: capped.x, y, width: capped.width, height: h };

            // проверим min_* ребёнка
            let cc = child.get_constraints();
            let min_w = cc.min_width.unwrap_or(1);
            let min_h = cc.min_height.unwrap_or(1);

            if rect.width < min_w || rect.height < min_h {
                Self::draw_overflow_text(align, buff, rect, "Overflowed");
            } else if let Err(e) = child.render(buff, rect) {
                // пробрасываем реальные ошибки детей
                return Err(e);
            }

            y = y.saturating_add(h);
        }
        Ok(())
    }

    fn _draw_horizontal(&mut self, capped: Rect, buff: &mut Buffer, sizes: Vec<(u16,u16)>) -> Result<(), AppError> {
        let align = self.alignment;
        let mut x = capped.x;

        for (i, child) in self.children.iter_mut().enumerate() {
            if x >= capped.x + capped.width { break; }

            let desired_w = sizes[i].0.max(1);
            let avail = (capped.x + capped.width).saturating_sub(x);
            let w = desired_w.min(avail);

            if w == 0 { break; }

            let rect = Rect { x, y: capped.y, width: w, height: capped.height };

            let cc = child.get_constraints();
            let min_w = cc.min_width.unwrap_or(1);
            let min_h = cc.min_height.unwrap_or(1);

            if rect.width < min_w || rect.height < min_h {
                Self::draw_overflow_text(align, buff, rect, "Overflowed");
            } else if let Err(e) = child.render(buff, rect) {
                return Err(e);
            }

            x = x.saturating_add(w);
        }
        Ok(())
    }

    fn _draw_grid(&mut self, capped: Rect, buff: &mut Buffer) -> Result<(), AppError> {
        let align = self.alignment;

        let child_sizes = match self.calculate_childs_size(capped) {
            Some(v) => v,
            None => return Ok(())
        };
        let heights: Vec<u16> = child_sizes.iter().map(|&(_, h)| h.max(1)).collect();
        if heights.is_empty() { return Ok(()); }

        let mut columns = Self::make_columns(&heights, capped.height);

        // если колонок больше, чем ширины, урежем — чтобы base_w >= 1
        if (columns.len() as u16) > capped.width {
            columns.truncate(capped.width as usize);
        }

        let n_cols = columns.len() as u16;
        if n_cols == 0 || capped.width == 0 || capped.height == 0 {
            // фоллбек: очень узко — вертикальный список
            let mut y = capped.y;
            for (idx, child) in self.children.iter_mut().enumerate() {
                if y >= capped.y + capped.height { break; }
                let h = child_sizes[idx].1.min((capped.y + capped.height).saturating_sub(y)).max(1);
                let rect = Rect { x: capped.x, y, width: capped.width, height: h };
                let cc = child.get_constraints();
                let min_w = cc.min_width.unwrap_or(1);
                let min_h = cc.min_height.unwrap_or(1);
                if rect.width < min_w || rect.height < min_h {
                    Self::draw_overflow_text(align, buff, rect, "Overflowed");
                } else if let Err(e) = child.render(buff, rect) {
                    return Err(e);
                }
                y = y.saturating_add(h);
            }
            return Ok(());
        }

        let base_w = capped.width / n_cols;
        let extra = capped.width % n_cols;

        if base_w == 0 {
            // фоллбек: вертикальный список
            let mut y = capped.y;
            for (idx, child) in self.children.iter_mut().enumerate() {
                if y >= capped.y + capped.height { break; }
                let h = child_sizes[idx].1.min((capped.y + capped.height).saturating_sub(y)).max(1);
                let rect = Rect { x: capped.x, y, width: capped.width, height: h };
                let cc = child.get_constraints();
                let min_w = cc.min_width.unwrap_or(1);
                let min_h = cc.min_height.unwrap_or(1);
                if rect.width < min_w || rect.height < min_h {
                    Self::draw_overflow_text(align, buff, rect, "Overflowed");
                } else if let Err(e) = child.render(buff, rect) {
                    return Err(e);
                }
                y = y.saturating_add(h);
            }
            return Ok(());
        }

        let mut x = capped.x;
        for (ci, col) in columns.into_iter().enumerate() {
            let w = base_w + if (ci as u16) < extra { 1 } else { 0 };
            let mut y = capped.y;

            for idx in col {
                if y >= capped.y + capped.height { break; }
                let h = child_sizes[idx].1.min((capped.y + capped.height).saturating_sub(y)).max(1);
                if h == 0 { break; }

                let rect = Rect { x, y, width: w, height: h };

                let child = &mut self.children[idx];
                let cc = child.get_constraints();
                let min_w = cc.min_width.unwrap_or(1);
                let min_h = cc.min_height.unwrap_or(1);

                if rect.width < min_w || rect.height < min_h {
                    Self::draw_overflow_text(align, buff, rect, "Overflowed");
                } else if let Err(e) = child.render(buff, rect) {
                    return Err(e);
                }

                y = y.saturating_add(h);
            }

            x = x.saturating_add(w);
        }

        Ok(())
    }
}

impl Widget for Container {

    fn render(&mut self, buff: &mut Buffer, area: Rect) -> Result<(), AppError> {
        // вместо фатальной ошибки — мягкий плейсхолдер
        if  area.width < 2 || area.height < 2 {
            if area.width > 0 && area.height > 0 {
                Self::draw_overflow_text(self.alignment, buff, area, "Overflowed");
            }
            return Ok(());
        }

        // применяем cap от self.width/self.height
        let mut capped = area;
        if let Some(cap_w) = self.width  { capped.width  = capped.width.min(cap_w); }
        if let Some(cap_h) = self.height { capped.height = capped.height.min(cap_h); }

        if self.children.is_empty() { return Ok(()); }

        let childs_size = match self.calculate_childs_size(capped) {
            Some(sz) => sz,
            None => return Ok(())
        };

        match self.layout {
            Layout::Vertical   => self._draw_vertical(capped, buff, childs_size)?,
            Layout::Horizontal => self._draw_horizontal(capped, buff, childs_size)?,
            Layout::Grid       => self._draw_grid(capped, buff)?,
        }

        Ok(())
    }

    // агрегируем констрейны ПО ЛЭЙАУТУ (а не суммой «как есть»)
    fn get_constraints(&self) -> ChildConstraints {
        fn opt_max(a: Option<u16>, b: Option<u16>) -> Option<u16> {
            match (a, b) {
                (Some(x), Some(y)) => Some(x.max(y)),
                (Some(x), None) | (None, Some(x)) => Some(x),
                (None, None) => None,
            }
        }
        fn opt_sum(a: Option<u16>, b: Option<u16>) -> Option<u16> {
            match (a, b) {
                (Some(x), Some(y)) => Some(x.saturating_add(y)),
                _ => None,
            }
        }

        let mut agg = match self.layout {
            Layout::Vertical => {
                // ширина = max по детям, высота = сумма по детям
                let mut min_w: Option<u16> = None;
                let mut max_w_all: Option<u16> = Some(0);
                let mut min_h_sum: u16 = 0;
                let mut max_h_sum: Option<u16> = Some(0);

                for ch in &self.children {
                    let c = ch.get_constraints();
                    min_w = opt_max(min_w, c.min_width);
                    max_w_all = match (max_w_all, c.max_width) {
                        (Some(acc), Some(w)) => Some(acc.max(w)),
                        _ => None,
                    };
                    min_h_sum = min_h_sum.saturating_add(c.min_height.unwrap_or(0));
                    max_h_sum = opt_sum(max_h_sum, c.max_height);
                }

                ChildConstraints {
                    min_width:  min_w,
                    max_width:  max_w_all,
                    min_height: Some(min_h_sum),
                    max_height: max_h_sum,
                }
            }
            Layout::Horizontal => {
                // ширина = сумма по детям, высота = max по детям
                let mut min_w_sum: u16 = 0;
                let mut max_w_sum: Option<u16> = Some(0);
                let mut min_h: Option<u16> = None;
                let mut max_h_all: Option<u16> = Some(0);

                for ch in &self.children {
                    let c = ch.get_constraints();
                    min_w_sum = min_w_sum.saturating_add(c.min_width.unwrap_or(0));
                    max_w_sum = opt_sum(max_w_sum, c.max_width);
                    min_h = opt_max(min_h, c.min_height);
                    max_h_all = match (max_h_all, c.max_height) {
                        (Some(acc), Some(h)) => Some(acc.max(h)),
                        _ => None,
                    };
                }

                ChildConstraints {
                    min_width:  Some(min_w_sum),
                    max_width:  max_w_sum,
                    min_height: min_h,
                    max_height: max_h_all,
                }
            }
            Layout::Grid => {
                // консервативно: размеры «плитки»
                let mut min_w: Option<u16> = None;
                let mut min_h: Option<u16> = None;
                for ch in &self.children {
                    let c = ch.get_constraints();
                    min_w = opt_max(min_w, c.min_width);
                    min_h = opt_max(min_h, c.min_height);
                }
                ChildConstraints {
                    min_width:  min_w,
                    max_width:  None,
                    min_height: min_h,
                    max_height: None,
                }
            }
        };

        if let Some(cap_w) = self.width {
            agg.max_width = agg.max_width.map(|mw| mw.min(cap_w)).or(Some(cap_w));
        }
        if let Some(cap_h) = self.height {
            agg.max_height = agg.max_height.map(|mh| mh.min(cap_h)).or(Some(cap_h));
        }

        agg
    }

    fn update(&mut self) -> Result<(), AppError> {
        for child in &mut self.children {
            child.update()?;
        }
        Ok(())
    }

    fn get_children(&self) -> &[Box<dyn Widget>] {
        &self.children
    }
}

