# Lab-2-Conway-s-Game-Of-Life

Juego de la Vida de Conway en Rust, utilizando un framebuffer lógico con renderizado en tiempo real sobre una ventana de `minifb`.  


# Controles

| Tecla | Acción |
|-------|--------|
| **ESPACIO** | Pausar / continuar |
| **R** | Población aleatoria |
| **C** | Limpiar tablero |
| **S** | Cambiar modo de bordes (DeadBorder / Torus) |
| **↑ / ↓** | Aumentar / disminuir velocidad |
| **P** | Insertar **Pulsar** al centro |
| **G** | Insertar **Glider** en la celda del mouse |
| **L** | Insertar **LWSS** en la celda del mouse |
| **T** | Cambiar tema visual (Classic / Aqua / Sunset / Neon) |
| **H** | Mostrar / ocultar grid |
| **B** | Activar / desactivar checkerboard de fondo |
| **V** | Activar / desactivar trails (estelas de células muertas) |


# Características

- Framebuffer lógico (`160x160`) escalado a una ventana de `900x900`.
- Funciones `point` y `get_color` implementadas para dibujar y consultar celdas.
- Algoritmo completo del **Game of Life**:
  - Underpopulation
  - Survival
  - Overpopulation
  - Reproduction
- Soporte para bordes DeadBorder y Torus (pantalla que se envuelve).
- Patrón inicial creativo con más de 10 organismos clásicos:
  - Block, Beehive, Loaf, Boat, Tub  
  - Blinker, Toad, Beacon, Pulsar  
  - Glider, Lightweight Spaceship (LWSS)
- Estilos visuales personalizables:
  - `Classic`: blanco/negro  
  - `Aqua`: azules y verdes suaves  
  - `Sunset`: tonos cálidos (naranja/fucsia)  
  - `Neon`: ciclo fluorescente


### Compilar y correr
```bash
cd lab2
cargo run
