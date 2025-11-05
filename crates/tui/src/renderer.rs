//! Renderizador TUI principal
//! 
//! Renderizador que usa crossterm para dibujar la interfaz en terminal,
//! manejando el ciclo de vida de la aplicación TUI.

use crossterm::{
    execute,
    terminal::{ClearType, EnterAlternateScreen, LeaveAlternateScreen},
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
};
use std::io::{stdout, Stdout};
use std::time::{Duration, Instant};
use thiserror::Error;

use crate::components::{Component, ComponentEvent, ComponentResult};
use crate::widgets::{Widget, Panel};

/// Error del renderizador TUI
#[derive(Error, Debug)]
pub enum TuiError {
    /// Error de inicialización
    #[error("Error de inicialización de TUI: {0}")]
    InitializationError(String),
    
    /// Error de renderizado
    #[error("Error de renderizado: {0}")]
    RenderError(String),
    
    /// Error de evento
    #[error("Error de evento: {0}")]
    EventError(String),
}

/// Configuración del renderizador
#[derive(Debug, Clone)]
pub struct TuiConfig {
    /// Frecuencia de actualización en FPS
    pub fps: u64,
    
    /// Timeout para eventos
    pub event_timeout: Duration,
    
    /// Limpiar pantalla en cada frame
    pub clear_on_render: bool,
    
    /// Mostrar cursor
    pub show_cursor: bool,
    
    /// Título de la ventana
    pub title: Option<String>,
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self {
            fps: 60,
            event_timeout: Duration::from_millis(16), // ~60 FPS
            clear_on_render: true,
            show_cursor: false,
            title: None,
        }
    }
}

/// Resultado de renderizado
pub type TuiResult<T> = Result<T, TuiError>;

/// Renderizador TUI principal
pub struct TuiRenderer {
    /// Configuración del renderizador
    config: TuiConfig,
    
    /// Panel principal
    root_panel: Panel,
    
    /// Componentes registrados
    components: Vec<Box<dyn Component>>,
    
    /// Componente enfocado
    focused_component: Option<usize>,
    
    /// Estado del renderizador
    is_running: bool,
    
    /// Última actualización
    last_update: Instant,
}

impl TuiRenderer {
    /// Crear nuevo renderizador
    pub fn new(config: TuiConfig) -> Self {
        let mut root_panel = Panel::new(80, 24); // Tamaño por defecto del terminal
        
        if let Some(title) = &config.title {
            root_panel = root_panel.with_title(title);
        }
        
        Self {
            config,
            root_panel,
            components: Vec::new(),
            focused_component: None,
            is_running: false,
            last_update: Instant::now(),
        }
    }
    
    /// Crear con configuración por defecto
    pub fn default() -> Self {
        Self::new(TuiConfig::default())
    }
    
    /// Inicializar TUI
    pub fn init(&mut self) -> TuiResult<()> {
        // Entrar en modo alternate screen
        execute!(stdout(), EnterAlternateScreen, EnableMouseCapture)
            .map_err(|e| TuiError::InitializationError(e.to_string()))?;
        
        // Configurar terminal si es necesario
        self.is_running = true;
        self.last_update = Instant::now();
        
        Ok(())
    }
    
    /// Cerrar TUI
    pub fn shutdown(&mut self) -> TuiResult<()> {
        // Salir del modo alternate screen
        execute!(stdout(), LeaveAlternateScreen, DisableMouseCapture)
            .map_err(|e| TuiError::InitializationError(e.to_string()))?;
        
        self.is_running = false;
        
        Ok(())
    }
    
    /// Registrar componente
    pub fn register_component(&mut self, component: Box<dyn Component>) {
        self.components.push(component);
        
        // Si es el primer componente, enfocarlo automáticamente
        if self.focused_component.is_none() {
            self.focused_component = Some(0);
        }
    }
    
    /// Obtener componente enfocado
    pub fn get_focused_component(&mut self) -> Option<&mut Box<dyn Component>> {
        if let Some(index) = self.focused_component {
            self.components.get_mut(index)
        } else {
            None
        }
    }
    
    /// Establecer componente enfocado
    pub fn set_focused_component(&mut self, index: usize) {
        if index < self.components.len() {
            // Desenfocar componente anterior
            if let Some(prev_index) = self.focused_component {
                if let Some(component) = self.components.get_mut(prev_index) {
                    component.set_focused(false);
                }
            }
            
            // Enfocar nuevo componente
            if let Some(component) = self.components.get_mut(index) {
                component.set_focused(true);
            }
            
            self.focused_component = Some(index);
        }
    }
    
    /// Renderizar frame completo
    pub fn render(&mut self) -> TuiResult<()> {
        if !self.is_running {
            return Ok(());
        }
        
        // Limpiar pantalla si está configurado
        if self.config.clear_on_render {
            execute!(stdout(), crossterm::terminal::Clear(ClearType::All))
                .map_err(|e| TuiError::RenderError(e.to_string()))?;
        }
        
        // Mover cursor a posición inicial
        if !self.config.show_cursor {
            execute!(stdout(), crossterm::cursor::MoveTo(0, 0))
                .map_err(|e| TuiError::RenderError(e.to_string()))?;
        }
        
        // Renderizar componentes
        for component in &self.components {
            let output = component.render();
            print!("{}", output);
        }
        
        // Forzar output
        stdout().flush()
            .map_err(|e| TuiError::RenderError(e.to_string()))?;
        
        self.last_update = Instant::now();
        
        Ok(())
    }
    
    /// Ejecutar bucle principal
    pub fn run(mut self) -> TuiResult<()> {
        self.init()?;
        
        while self.is_running {
            // Procesar eventos
            if event::poll(self.config.event_timeout)
                .map_err(|e| TuiError::EventError(e.to_string()))? {
                
                match event::read()
                    .map_err(|e| TuiError::EventError(e.to_string()))? {
                    
                    Event::Key(key_event) => {
                        self.handle_key_event(key_event);
                    }
                    
                    Event::Mouse(mouse_event) => {
                        self.handle_mouse_event(mouse_event);
                    }
                    
                    Event::Resize(width, height) => {
                        self.handle_resize(width, height);
                    }
                }
            }
            
            // Renderizar
            self.render()?;
        }
        
        self.shutdown()?;
        
        Ok(())
    }
    
    /// Manejar eventos de teclado
    fn handle_key_event(&mut self, key_event: crossterm::event::KeyEvent) {
        match key_event.code {
            // Manejo de focus entre componentes
            KeyCode::Tab => {
                if !self.components.is_empty() {
                    let next_index = if let Some(current) = self.focused_component {
                        (current + 1) % self.components.len()
                    } else {
                        0
                    };
                    self.set_focused_component(next_index);
                }
            }
            
            KeyCode::BackTab => {
                if !self.components.is_empty() {
                    let prev_index = if let Some(current) = self.focused_component {
                        if current == 0 {
                            self.components.len() - 1
                        } else {
                            current - 1
                        }
                    } else {
                        0
                    };
                    self.set_focused_component(prev_index);
                }
            }
            
            // Salir de la aplicación
            KeyCode::Esc => {
                if key_event.modifiers == crossterm::event::KeyModifiers::NONE {
                    self.is_running = false;
                }
            }
            
            // Eventos específicos del componente
            _ => {
                if let Some(component) = self.get_focused_component() {
                    let event = crossterm::event::Event::Key(key_event);
                    let result = component.handle_event(event);
                    
                    self.handle_component_result(result);
                }
            }
        }
    }
    
    /// Manejar eventos de mouse
    fn handle_mouse_event(&mut self, _mouse_event: crossterm::event::MouseEvent) {
        // TODO: Implementar manejo de mouse
        // Por ahora solo actualizar posición o hacer click
    }
    
    /// Manejar cambios de tamaño
    fn handle_resize(&mut self, width: u16, height: u16) {
        // Actualizar tamaño del panel raíz
        self.root_panel.width = width as usize;
        self.root_panel.height = height as usize;
        
        // Notificar a componentes del cambio de tamaño
        for component in &mut self.components {
            // TODO: Implementar notificación de resize
        }
    }
    
    /// Manejar resultado de componente
    fn handle_component_result(&mut self, result: ComponentResult) {
        match result.event {
            ComponentEvent::Cancel => {
                self.is_running = false;
            }
            
            ComponentEvent::Activate => {
                // Componente activado - podría cambiar focus o ejecutar acción
            }
            
            ComponentEvent::Custom(event_type) => {
                match event_type.as_str() {
                    "next_component" => {
                        if !self.components.is_empty() {
                            let next_index = if let Some(current) = self.focused_component {
                                (current + 1) % self.components.len()
                            } else {
                                0
                            };
                            self.set_focused_component(next_index);
                        }
                    }
                    
                    "prev_component" => {
                        if !self.components.is_empty() {
                            let prev_index = if let Some(current) = self.focused_component {
                                if current == 0 {
                                    self.components.len() - 1
                                } else {
                                    current - 1
                                }
                            } else {
                                0
                            };
                            self.set_focused_component(prev_index);
                        }
                    }
                    
                    _ => {
                        // Eventos personalizados pueden ser manejados aquí
                    }
                }
            }
            
            _ => {
                // Otros eventos pueden ser manejados aquí
            }
        }
    }
    
    /// Obtener estado del renderizador
    pub fn is_running(&self) -> bool {
        self.is_running
    }
    
    /// Forzar salida
    pub fn quit(&mut self) {
        self.is_running = false;
    }
    
    /// Agregar widget al panel raíz
    pub fn add_widget<T: Widget + 'static>(&mut self, widget: T) {
        self.root_panel.add_widget(widget);
    }
    
    /// Obtener dimensiones actuales
    pub fn get_size(&self) -> (usize, usize) {
        (self.root_panel.height, self.root_panel.width)
    }
}

impl Drop for TuiRenderer {
    fn drop(&mut self) {
        if self.is_running {
            // Intentar cerrar limpiamente
            let _ = self.shutdown();
        }
    }
}

/// Ejecutor TUI simplificado
pub struct TuiApp {
    renderer: TuiRenderer,
}

impl TuiApp {
    /// Crear nueva aplicación TUI
    pub fn new(config: TuiConfig) -> Self {
        let renderer = TuiRenderer::new(config);
        Self { renderer }
    }
    
    /// Crear con configuración por defecto
    pub fn default() -> Self {
        Self::new(TuiConfig::default())
    }
    
    /// Registrar componente
    pub fn register_component(&mut self, component: Box<dyn Component>) {
        self.renderer.register_component(component);
    }
    
    /// Ejecutar aplicación
    pub fn run(&mut self) -> TuiResult<()> {
        self.renderer.run()
    }
    
    /// Obtener renderizador
    pub fn renderer(&mut self) -> &mut TuiRenderer {
        &mut self.renderer
    }
}

/// Builder para configuración TUI
pub struct TuiConfigBuilder {
    config: TuiConfig,
}

impl TuiConfigBuilder {
    /// Crear nuevo builder
    pub fn new() -> Self {
        Self {
            config: TuiConfig::default(),
        }
    }
    
    /// Establecer FPS
    pub fn fps(mut self, fps: u64) -> Self {
        self.config.fps = fps;
        self
    }
    
    /// Establecer timeout de eventos
    pub fn event_timeout(mut self, timeout: Duration) -> Self {
        self.config.event_timeout = timeout;
        self
    }
    
    /// Habilitar/deshabilitar limpieza de pantalla
    pub fn clear_on_render(mut self, clear: bool) -> Self {
        self.config.clear_on_render = clear;
        self
    }
    
    /// Mostrar/ocultar cursor
    pub fn show_cursor(mut self, show: bool) -> Self {
        self.config.show_cursor = show;
        self
    }
    
    /// Establecer título
    pub fn title<T: Into<String>>(mut self, title: T) -> Self {
        self.config.title = Some(title.into());
        self
    }
    
    /// Construir configuración
    pub fn build(self) -> TuiConfig {
        self.config
    }
}

impl Default for TuiConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}