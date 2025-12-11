# StartWin 🚀

Un launcher de aplicaciones moderno y personalizable para Windows que reemplaza el menú de inicio por defecto con una interfaz rápida, inteligente y potente.

![Version](https://img.shields.io/badge/version-1.1.0-blue)
![Rust](https://img.shields.io/badge/rust-1.70+-orange)
![License](https://img.shields.io/badge/license-MIT-green)

## ✨ Características

### 🔍 Búsqueda Inteligente

- **Búsqueda Fuzzy**: Encuentra aplicaciones incluso con errores de tipeo
- **Coincidencias por iniciales**: "chr" encuentra "Google Chrome"
- **Búsqueda tolerante**: No necesitas escribir el nombre exacto

### 🧮 Calculadora Integrada

- **Evaluación instantánea**: Escribe expresiones matemáticas y obtén el resultado al instante
- **Operaciones soportadas**: `+`, `-`, `*`, `/`, `^` (potencia), paréntesis
- **Ejemplos**:
  - `2+2` → 4
  - `(10*5)/2` → 25
  - `2^3` → 8

### ⌨️ Atajos de Teclado Avanzados

- **Enter**: Lanzar aplicación seleccionada
- **Alt+Enter**: Ejecutar como administrador
- **Shift+Enter**: Abrir ubicación del archivo
- **Win/⊞**: Abrir/cerrar StartWin
- **Esc**: Cerrar el launcher
- **↑/↓**: Navegar por las aplicaciones

### 🎨 Interfaz Moderna

- **Soporte de modo oscuro**: Se adapta automáticamente al tema de Windows
- **Esquinas redondeadas**: Diseño moderno con DWM
- **Sistema de bandeja**: Icono en la bandeja del sistema
- **Sin ventana de consola**: Experiencia GUI limpia

### 📊 Aprendizaje Inteligente

- **Historial de uso**: Las aplicaciones más usadas aparecen primero
- **Base de datos SQLite**: Almacena estadísticas de uso
- **Ordenamiento inteligente**: Combina frecuencia de uso y orden alfabético

## 🚀 Instalación

### Desde el Código Fuente

**Requisitos:**

- Rust 1.70 o superior
- Windows 10/11

```bash
# Clonar el repositorio
git clone https://github.com/tuusuario/startwin.git
cd startwin

# Compilar en modo release
cargo build --release

# El ejecutable estará en target/release/startwin.exe
```

### Uso

1. Ejecuta `startwin.exe`
2. La aplicación se iniciará en la bandeja del sistema
3. Presiona la tecla **Win** (⊞) o haz clic en el botón de inicio de Windows
4. ¡StartWin aparecerá en lugar del menú de inicio!

## 📖 Guía de Uso

### Búsqueda de Aplicaciones

```
Escribe en el cuadro de búsqueda:
├─ "chrome" → Google Chrome, Chrome Remote Desktop...
├─ "vs code" → Visual Studio Code
├─ "calc" → Calculator, Calendar...
└─ "chr" → Google Chrome (búsqueda por iniciales)
```

### Calculadora

```
Expresiones matemáticas:
├─ 2+2 → = 4
├─ 100/4 → = 25
├─ (3+4)*2 → = 14
├─ 2^10 → = 1024
└─ 15.5 * 2 → = 31
```

### Atajos de Lanzamiento

| Atajo           | Acción                       |
| --------------- | ---------------------------- |
| **Enter**       | Abrir aplicación normalmente |
| **Alt+Enter**   | Ejecutar como administrador  |
| **Shift+Enter** | Abrir ubicación del archivo  |
| **Doble clic**  | Abrir aplicación             |

## 🏗️ Arquitectura

El proyecto está organizado en módulos especializados:

```
src/
├── main.rs           # Punto de entrada y lógica principal
├── app_model.rs      # Modelo de datos y gestor de aplicaciones
├── calculator.rs     # Motor de evaluación matemática
├── db.rs             # Gestión de base de datos SQLite
├── hooks.rs          # Hooks de teclado y ratón
├── scanner.rs        # Escaneo de aplicaciones del sistema
├── ui.rs             # Componentes de interfaz de usuario
└── utils.rs          # Funciones de utilidad
```

## 🧪 Testing

El proyecto incluye 22 tests unitarios que cubren todos los módulos:

```bash
# Ejecutar todos los tests
cargo test

# Ejecutar tests con output detallado
cargo test -- --nocapture

# Ejecutar un test específico
cargo test test_fuzzy_search
```

## 🔧 Desarrollo

### Compilación en Modo Debug

```bash
cargo build
```

### Compilación en Modo Release

```bash
cargo build --release
```

### Formato de Código

```bash
cargo fmt
```

### Linting

```bash
cargo clippy
```

## 📦 Dependencias

- **windows** (0.52.0): Bindings de Windows API
- **rusqlite** (0.31.0): Base de datos SQLite
- **once_cell** (1.19): Lazy static initialization
- **fuzzy-matcher** (0.3): Algoritmo de búsqueda fuzzy
- **meval** (0.2): Evaluador de expresiones matemáticas

## 🗺️ Roadmap

### Versión 1.2 (Próximamente)

- [ ] Aplicaciones favoritas/pinned
- [ ] Temas personalizables
- [ ] Historial de búsquedas recientes
- [ ] Configuración de tamaño de ventana

### Versión 1.3

- [ ] Búsqueda de archivos y carpetas
- [ ] Categorización de aplicaciones
- [ ] Web search integrado
- [ ] Sugerencias contextuales

### Versión 2.0

- [ ] Sistema de plugins
- [ ] Sincronización multi-PC
- [ ] Perfiles múltiples
- [ ] API para desarrolladores

## 🤝 Contribuir

Las contribuciones son bienvenidas! Por favor:

1. Fork el proyecto
2. Crea una rama para tu feature (`git checkout -b feature/AmazingFeature`)
3. Commit tus cambios (`git commit -m 'Add some AmazingFeature'`)
4. Push a la rama (`git push origin feature/AmazingFeature`)
5. Abre un Pull Request

## 📝 Changelog

### v1.1.0 (2025-12-11)

- ✨ Búsqueda fuzzy inteligente
- 🧮 Calculadora integrada
- ⌨️ Alt+Enter para ejecutar como admin
- 📁 Shift+Enter para abrir ubicación
- 🏗️ Refactorización modular completa

### v1.0.0 (2025-12-10)

- 🎉 Lanzamiento inicial
- 🔍 Búsqueda básica de aplicaciones
- 🎨 Soporte de modo oscuro
- 📊 Tracking de uso de aplicaciones
- ⊞ Interceptación del botón de inicio

## 📄 Licencia

Este proyecto está bajo la Licencia MIT. Ver el archivo `LICENSE` para más detalles.

## 🙏 Agradecimientos

- Gracias a la comunidad de Rust por las excelentes librerías
- Microsoft por la Windows API
- Todos los contribuidores y testers

## 💬 Contacto

- **Issues**: [GitHub Issues](https://github.com/tuusuario/startwin/issues)
- **Discussions**: [GitHub Discussions](https://github.com/tuusuario/startwin/discussions)

---

**⭐ Si te gusta StartWin, dale una estrella en GitHub!**
