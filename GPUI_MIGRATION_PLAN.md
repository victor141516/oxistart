# Plan de Migración a GPUI

## Visión General

Migrar Oxistart de Win32 API nativa a GPUI (Zed's GPU-accelerated UI framework).

## Desafíos Principales

### 1. Arquitectura Actual vs GPUI

**Actual:**

- Win32 API directa para UI (CreateWindowExW, ListView, etc.)
- Hooks de bajo nivel para capturar teclas Windows
- Integración directa con sistema de iconos de Windows
- Bandeja del sistema con Win32

**GPUI:**

- Framework declarativo moderno con componentes
- Sistema de eventos integrado
- Aceleración GPU para renderizado
- Cross-platform (aunque solo necesitamos Windows)

### 2. Funcionalidades Críticas a Preservar

- ✅ Reemplazo del botón de inicio (requiere hooks Win32)
- ✅ Captura de tecla Windows global
- ✅ Icono en bandeja del sistema
- ✅ Ventana siempre encima (topmost)
- ✅ Búsqueda fuzzy
- ✅ Calculadora integrada
- ✅ Atajos de teclado (Alt+Enter, Shift+Enter)
- ✅ Iconos del sistema para aplicaciones
- ✅ Dark mode nativo

## Fases de Migración

### Fase 1: Setup y Arquitectura Base

1. Añadir GPUI y dependencias
2. Crear estructura híbrida (mantener hooks Win32)
3. Setup básico de ventana GPUI
4. Integrar sistema de eventos

### Fase 2: Componentes UI Core

1. Input de búsqueda (TextInput)
2. Lista de resultados (ScrollView + List)
3. Label de calculadora
4. Gestión de estado con GPUI

### Fase 3: Integración con Sistema

1. Mantener hooks Win32 (keyboard_hook, mouse_hook)
2. Puente entre hooks y eventos GPUI
3. Gestión de ventana topmost
4. Bandeja del sistema (mantener Win32)

### Fase 4: Estilos y Tema

1. Sistema de colores (light/dark)
2. Tipografía (Segoe UI)
3. Espaciado y layout
4. Animaciones suaves

### Fase 5: Testing y Refinamiento

1. Testing funcional completo
2. Optimización de rendimiento
3. Gestión de memoria
4. Debugging

## Estructura de Código Propuesta

```
src/
├── main.rs              # Entry point, setup GPUI app
├── app.rs               # App state y lógica principal
├── components/
│   ├── mod.rs
│   ├── search_input.rs  # Componente de búsqueda
│   ├── result_list.rs   # Lista de resultados
│   └── calculator.rs    # Display de calculadora
├── state/
│   ├── mod.rs
│   └── app_state.rs     # Estado global de la app
├── hooks/               # Mantener Win32 hooks
│   ├── mod.rs
│   ├── keyboard.rs
│   └── mouse.rs
├── ui/                  # GPUI UI específico
│   ├── mod.rs
│   ├── theme.rs
│   └── styles.rs
├── app_model.rs         # Mantener
├── calculator.rs        # Mantener
├── db.rs               # Mantener
├── scanner.rs          # Mantener
├── settings.rs         # Mantener
└── utils.rs            # Mantener/adaptar
```

## Consideraciones Técnicas

### Gestión de Estado

GPUI usa un modelo de estado reactivo. Necesitaremos:

- `Model<T>` para estado compartido
- `View<T>` para componentes de UI
- `WindowContext` para operaciones de ventana

### Integración Win32

Los hooks de Windows se mantendrán, pero necesitamos:

1. Un puente entre callbacks Win32 y eventos GPUI
2. Thread-safe communication (canales o Arc<Mutex>)
3. Dispatch de eventos al contexto GPUI

### Rendimiento

- GPUI usa GPU para renderizado = más fluido
- Pero añade complejidad
- Cuidado con re-renderizado excesivo

## Próximos Pasos

1. ✅ Crear branch `feature/gpui-migration`
2. Añadir GPUI a Cargo.toml
3. Crear estructura básica de app GPUI
4. Implementar componente de búsqueda simple
5. Probar integración básica

## Recursos

- GPUI docs: https://www.gpui.rs/
- Zed source: https://github.com/zed-industries/zed
- Examples: https://github.com/zed-industries/zed/tree/main/crates/gpui/examples
