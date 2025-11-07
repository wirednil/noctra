//! Gestor de layout para TUI
//! 
//! Sistema de layout para organizar componentes en pantalla
//! con diferentes estrategias de posicionamiento y redimensionado.

use crate::components::Component;
use crate::widgets::{Widget, Panel};
use std::collections::HashMap;

/// Posición de un elemento
#[derive(Debug, Clone, Copy)]
pub struct Position {
    /// Coordenada X
    pub x: usize,
    
    /// Coordenada Y
    pub y: usize,
}

/// Dimensiones de un elemento
#[derive(Debug, Clone, Copy)]
pub struct Size {
    /// Ancho
    pub width: usize,
    
    /// Alto
    pub height: usize,
}

/// Rectángulo de layout
#[derive(Debug, Clone, Copy)]
pub struct Rect {
    /// Posición superior izquierda
    pub position: Position,

    /// Dimensiones
    pub size: Size,
}

impl Rect {
    /// Crear nuevo rectángulo
    pub fn new(x: usize, y: usize, width: usize, height: usize) -> Self {
        Self {
            position: Position { x, y },
            size: Size { width, height },
        }
    }
    
    /// Verificar si contiene un punto
    pub fn contains(&self, x: usize, y: usize) -> bool {
        x >= self.position.x && 
        x < self.position.x + self.size.width &&
        y >= self.position.y && 
        y < self.position.y + self.size.height
    }
    
    /// Verificar intersección con otro rectángulo
    pub fn intersects(&self, other: &Rect) -> bool {
        !(self.position.x + self.size.width <= other.position.x ||
          other.position.x + other.size.width <= self.position.x ||
          self.position.y + self.size.height <= other.position.y ||
          other.position.y + other.size.height <= self.position.y)
    }
    
    /// Calcular área
    pub fn area(&self) -> usize {
        self.size.width * self.size.height
    }
}

/// Estrategia de layout
#[derive(Debug, Clone)]
pub enum LayoutStrategy {
    /// Layout vertical fijo
    Vertical {
        /// Espaciado entre elementos
        spacing: usize,
        
        /// Padding general
        padding: usize,
        
        /// Elementos de tamaño fijo
        fixed_heights: HashMap<String, usize>,
    },
    
    /// Layout horizontal fijo
    Horizontal {
        /// Espaciado entre elementos
        spacing: usize,
        
        /// Padding general
        padding: usize,
        
        /// Elementos de ancho fijo
        fixed_widths: HashMap<String, usize>,
    },
    
    /// Grid con columnas y filas específicas
    Grid {
        /// Número de columnas
        columns: usize,
        
        /// Número de filas
        rows: usize,
        
        /// Espaciado entre columnas
        column_spacing: usize,
        
        /// Espaciado entre filas
        row_spacing: usize,
        
        /// Padding general
        padding: usize,
        
        /// Posición de elementos específicos
        positions: HashMap<String, (usize, usize)>, // (row, col)
        
        /// Span de elementos (para ocupar múltiples celdas)
        spans: HashMap<String, (usize, usize)>, // (row_span, col_span)
    },
    
    /// Layout absoluto (posiciones específicas)
    Absolute {
        /// Posiciones de elementos
        positions: HashMap<String, Rect>,
    },
    
    /// Layout con peso proporcional
    Weighted {
        /// Peso para el eje vertical
        vertical_weights: HashMap<String, f64>,
        
        /// Peso para el eje horizontal
        horizontal_weights: HashMap<String, f64>,
        
        /// Espaciado entre elementos
        spacing: usize,
        
        /// Padding general
        padding: usize,
    },
}

/// Elemento de layout
pub struct LayoutElement {
    /// Identificador único
    pub id: String,

    /// Componente asociado
    pub component: Box<dyn Component>,
    
    /// Rectángulo de destino
    pub rect: Rect,
    
    /// Indica si el elemento debe ser visible
    pub visible: bool,
    
    /// Indica si el elemento puede ser redimensionado
    pub resizable: bool,
    
    /// Restricciones de tamaño mínimo
    pub min_size: Option<Size>,
    
    /// Restricciones de tamaño máximo
    pub max_size: Option<Size>,
}

impl LayoutElement {
    /// Crear nuevo elemento de layout
    pub fn new<T: Component + 'static>(id: String, component: T) -> Self {
        let (width, height) = component.get_size();
        
        Self {
            id,
            component: Box::new(component),
            rect: Rect::new(0, 0, width, height),
            visible: true,
            resizable: true,
            min_size: None,
            max_size: None,
        }
    }
    
    /// Crear con rectángulo específico
    pub fn with_rect<T: Component + 'static>(id: String, component: T, rect: Rect) -> Self {
        Self {
            id,
            component: Box::new(component),
            rect,
            visible: true,
            resizable: true,
            min_size: None,
            max_size: None,
        }
    }
    
    /// Establecer visibilidad
    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }
    
    /// Establecer si es redimensionable
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }
    
    /// Establecer tamaño mínimo
    pub fn min_size(mut self, min_size: Size) -> Self {
        self.min_size = Some(min_size);
        self
    }
    
    /// Establecer tamaño máximo
    pub fn max_size(mut self, max_size: Size) -> Self {
        self.max_size = Some(max_size);
        self
    }
}

/// Gestor de layout principal
pub struct LayoutManager {
    /// Elementos del layout
    elements: Vec<LayoutElement>,
    
    /// Estrategia de layout actual
    strategy: LayoutStrategy,
    
    /// Área total disponible
    total_area: Rect,
    
    /// Padding de los bordes
    border_padding: usize,
}

impl LayoutManager {
    /// Crear nuevo gestor
    pub fn new(total_width: usize, total_height: usize) -> Self {
        let strategy = LayoutStrategy::Vertical {
            spacing: 1,
            padding: 1,
            fixed_heights: HashMap::new(),
        };
        
        Self {
            elements: Vec::new(),
            strategy,
            total_area: Rect::new(0, 0, total_width, total_height),
            border_padding: 1,
        }
    }
    
    /// Crear con estrategia específica
    pub fn with_strategy(total_width: usize, total_height: usize, strategy: LayoutStrategy) -> Self {
        Self {
            elements: Vec::new(),
            strategy,
            total_area: Rect::new(0, 0, total_width, total_height),
            border_padding: 1,
        }
    }
    
    /// Agregar elemento
    pub fn add_element<T: Component + 'static>(&mut self, id: String, component: T) {
        let element = LayoutElement::new(id, component);
        self.elements.push(element);
        self.recalculate_layout();
    }
    
    /// Agregar elemento con rectángulo específico
    pub fn add_element_with_rect<T: Component + 'static>(&mut self, id: String, component: T, rect: Rect) {
        let element = LayoutElement::with_rect(id, component, rect);
        self.elements.push(element);
    }
    
    /// Remover elemento
    pub fn remove_element(&mut self, id: &str) {
        self.elements.retain(|element| element.id != id);
        self.recalculate_layout();
    }
    
    /// Obtener elemento por ID
    pub fn get_element(&mut self, id: &str) -> Option<&mut LayoutElement> {
        self.elements.iter_mut().find(|element| element.id == id)
    }
    
    /// Obtener todos los elementos visibles
    pub fn get_visible_elements(&mut self) -> Vec<&mut LayoutElement> {
        self.elements.iter_mut()
            .filter(|element| element.visible)
            .collect()
    }
    
    /// Actualizar estrategia de layout
    pub fn set_strategy(&mut self, strategy: LayoutStrategy) {
        self.strategy = strategy;
        self.recalculate_layout();
    }
    
    /// Actualizar área total
    pub fn update_total_area(&mut self, width: usize, height: usize) {
        self.total_area = Rect::new(0, 0, width, height);
        self.recalculate_layout();
    }
    
    /// Recalcular layout completo
    pub fn recalculate_layout(&mut self) {
        // Clonar la estrategia para evitar conflictos de préstamos
        let strategy = self.strategy.clone();
        match strategy {
            LayoutStrategy::Vertical { spacing, padding, ref fixed_heights } => {
                self.apply_vertical_layout(spacing, padding, fixed_heights);
            }

            LayoutStrategy::Horizontal { spacing, padding, ref fixed_widths } => {
                self.apply_horizontal_layout(spacing, padding, fixed_widths);
            }

            LayoutStrategy::Grid { columns, rows, column_spacing, row_spacing, padding, ref positions, ref spans } => {
                self.apply_grid_layout(columns, rows, column_spacing, row_spacing, padding, positions, spans);
            }

            LayoutStrategy::Absolute { ref positions } => {
                self.apply_absolute_layout(positions);
            }

            LayoutStrategy::Weighted { ref vertical_weights, ref horizontal_weights, spacing, padding } => {
                self.apply_weighted_layout(vertical_weights, horizontal_weights, spacing, padding);
            }
        }
    }
    
    /// Aplicar layout vertical
    fn apply_vertical_layout(&mut self, spacing: usize, padding: usize, fixed_heights: &HashMap<String, usize>) {
        let mut current_y = self.total_area.position.y + padding;
        let available_width = self.total_area.size.width.saturating_sub(padding * 2);
        
        for element in &mut self.elements {
            if !element.visible {
                continue;
            }
            
            // Usar altura fija si está definida, sino calcular
            let height = if let Some(fixed_height) = fixed_heights.get(&element.id) {
                *fixed_height
            } else {
                let (_, component_height) = element.component.get_size();
                component_height
            };
            
            // Aplicar restricciones de tamaño
            let final_height = if let Some(min_size) = element.min_size {
                height.max(min_size.height)
            } else {
                height
            };
            
            let final_height = if let Some(max_size) = element.max_size {
                final_height.min(max_size.height)
            } else {
                final_height
            };
            
            // Calcular posición y tamaño
            element.rect.position.x = self.total_area.position.x + padding;
            element.rect.position.y = current_y;
            element.rect.size.width = available_width;
            element.rect.size.height = final_height;
            
            current_y += final_height + spacing;
        }
    }
    
    /// Aplicar layout horizontal
    fn apply_horizontal_layout(&mut self, spacing: usize, padding: usize, fixed_widths: &HashMap<String, usize>) {
        let mut current_x = self.total_area.position.x + padding;
        let available_height = self.total_area.size.height.saturating_sub(padding * 2);
        
        for element in &mut self.elements {
            if !element.visible {
                continue;
            }
            
            // Usar ancho fijo si está definido, sino calcular
            let width = if let Some(fixed_width) = fixed_widths.get(&element.id) {
                *fixed_width
            } else {
                let (component_width, _) = element.component.get_size();
                component_width
            };
            
            // Aplicar restricciones de tamaño
            let final_width = if let Some(min_size) = element.min_size {
                width.max(min_size.width)
            } else {
                width
            };
            
            let final_width = if let Some(max_size) = element.max_size {
                final_width.min(max_size.width)
            } else {
                final_width
            };
            
            // Calcular posición y tamaño
            element.rect.position.x = current_x;
            element.rect.position.y = self.total_area.position.y + padding;
            element.rect.size.width = final_width;
            element.rect.size.height = available_height;
            
            current_x += final_width + spacing;
        }
    }
    
    /// Aplicar layout de grid
    fn apply_grid_layout(
        &mut self, 
        columns: usize, 
        rows: usize, 
        column_spacing: usize, 
        row_spacing: usize, 
        padding: usize,
        positions: &HashMap<String, (usize, usize)>,
        spans: &HashMap<String, (usize, usize)>
    ) {
        let cell_width = (self.total_area.size.width.saturating_sub(padding * 2 + column_spacing * (columns - 1))) / columns;
        let cell_height = (self.total_area.size.height.saturating_sub(padding * 2 + row_spacing * (rows - 1))) / rows;
        
        for element in &mut self.elements {
            if !element.visible {
                continue;
            }
            
            let (row, col) = positions.get(&element.id).unwrap_or(&(0, 0));
            let (row_span, col_span) = spans.get(&element.id).unwrap_or(&(1, 1));
            
            // Calcular posición
            element.rect.position.x = self.total_area.position.x + padding + col * (cell_width + column_spacing);
            element.rect.position.y = self.total_area.position.y + padding + row * (cell_height + row_spacing);
            
            // Calcular tamaño
            element.rect.size.width = col_span * cell_width + (col_span - 1) * column_spacing;
            element.rect.size.height = row_span * cell_height + (row_span - 1) * row_spacing;
        }
    }
    
    /// Aplicar layout absoluto
    fn apply_absolute_layout(&mut self, positions: &HashMap<String, Rect>) {
        for element in &mut self.elements {
            if let Some(rect) = positions.get(&element.id) {
                element.rect = *rect;
            }
        }
    }
    
    /// Aplicar layout ponderado
    fn apply_weighted_layout(
        &mut self, 
        vertical_weights: &HashMap<String, f64>, 
        horizontal_weights: &HashMap<String, f64>, 
        spacing: usize, 
        padding: usize
    ) {
        let total_vertical_weight: f64 = vertical_weights.values().sum();
        let total_horizontal_weight: f64 = horizontal_weights.values().sum();
        
        if total_vertical_weight == 0.0 || total_horizontal_weight == 0.0 {
            // Fallback a layout vertical si no hay pesos válidos
            return self.apply_vertical_layout(spacing, padding, &HashMap::new());
        }
        
        let available_width = self.total_area.size.width.saturating_sub(padding * 2);
        let available_height = self.total_area.size.height.saturating_sub(padding * 2);

        for (_i, element) in self.elements.iter_mut().enumerate() {
            if !element.visible {
                continue;
            }

            let v_weight = vertical_weights.get(&element.id).unwrap_or(&1.0);
            let h_weight = horizontal_weights.get(&element.id).unwrap_or(&1.0);

            let width = (available_width as f64 * h_weight / total_horizontal_weight) as usize;
            let height = (available_height as f64 * v_weight / total_vertical_weight) as usize;

            element.rect.size.width = width;
            element.rect.size.height = height;
        }
    }
    
    /// Renderizar todos los elementos visibles
    pub fn render(&mut self) -> String {
        let mut output = String::new();
        
        for element in &self.elements {
            if element.visible {
                // TODO: Posicionar cursor y renderizar componente en su posición
                let component_output = element.component.render();
                output.push_str(&component_output);
                output.push('\n');
            }
        }
        
        output
    }
    
    /// Obtener área total
    pub fn get_total_area(&self) -> Rect {
        self.total_area
    }
    
    /// Verificar si un punto está dentro de algún elemento
    pub fn hit_test(&self, x: usize, y: usize) -> Option<&String> {
        for element in &self.elements {
            if element.visible && element.rect.contains(x, y) {
                return Some(&element.id);
            }
        }
        None
    }
}

/// Builder para LayoutManager
pub struct LayoutBuilder {
    strategy: LayoutStrategy,
    total_width: usize,
    total_height: usize,
    border_padding: usize,
}

impl LayoutBuilder {
    /// Crear nuevo builder
    pub fn new(total_width: usize, total_height: usize) -> Self {
        Self {
            strategy: LayoutStrategy::Vertical {
                spacing: 1,
                padding: 1,
                fixed_heights: HashMap::new(),
            },
            total_width,
            total_height,
            border_padding: 1,
        }
    }
    
    /// Configurar estrategia vertical
    pub fn vertical(mut self, spacing: usize, padding: usize) -> Self {
        self.strategy = LayoutStrategy::Vertical {
            spacing,
            padding,
            fixed_heights: HashMap::new(),
        };
        self
    }
    
    /// Configurar estrategia horizontal
    pub fn horizontal(mut self, spacing: usize, padding: usize) -> Self {
        self.strategy = LayoutStrategy::Horizontal {
            spacing,
            padding,
            fixed_widths: HashMap::new(),
        };
        self
    }
    
    /// Configurar estrategia de grid
    pub fn grid(mut self, columns: usize, rows: usize, column_spacing: usize, row_spacing: usize, padding: usize) -> Self {
        self.strategy = LayoutStrategy::Grid {
            columns,
            rows,
            column_spacing,
            row_spacing,
            padding,
            positions: HashMap::new(),
            spans: HashMap::new(),
        };
        self
    }
    
    /// Configurar estrategia absoluta
    pub fn absolute(mut self) -> Self {
        self.strategy = LayoutStrategy::Absolute {
            positions: HashMap::new(),
        };
        self
    }
    
    /// Configurar estrategia ponderada
    pub fn weighted(mut self, spacing: usize, padding: usize) -> Self {
        self.strategy = LayoutStrategy::Weighted {
            vertical_weights: HashMap::new(),
            horizontal_weights: HashMap::new(),
            spacing,
            padding,
        };
        self
    }
    
    /// Establecer padding de borde
    pub fn border_padding(mut self, padding: usize) -> Self {
        self.border_padding = padding;
        self
    }
    
    /// Construir LayoutManager
    pub fn build(self) -> LayoutManager {
        LayoutManager::with_strategy(self.total_width, self.total_height, self.strategy)
    }
}