# shell

`shell` es un **shell manager** para macOS: crea VMs Linux con [Lima](https://lima-vm.io/)
y te permite saltar entre el host y varios entornos manteniendo **los mismos
archivos** del host, pero aplicando **configuración global distinta** en cada uno.

Es el modelo de un *devcontainer*, con una diferencia: cada shell tiene su propio
home y su propio perfil (dotfiles), mientras que el directorio
del host que elijas se montan dentro de todas las VMs de forma escritura.

```
host (macOS)                shell A (Ubuntu)         shell B (Ubuntu)
~/Workspaces/project    ──►  ~/Workspace            (mismo dir, en vivo)
~/.zshrc (macOS)             ~/.zshrc (A)             ~/.zshrc (B)          (distintos por shell)
                             node@18, python@3.11     node@20, rust
```

## Comandos

```sh
shell create <name> --workspace ~/dir [--image ubuntu] [--cpus 8] [--memory 8GiB]
                    [--disk 40GiB]
shell enter  <name>      # abre una sesión interactiva (SSH) en la VM
shell remove <name>      # borra perfil + VM + dotfiles
shell list [name]        # estado de todos los shells, o detalle de uno
shell start|stop <name>  # ciclo de vida de la VM
```

`--workspace` es **obligatorio** al crear un shell. La carpeta del host aparece
**dentro del guest en `~/Workspace`** (el nombre que tenga en el host da igual).
Al crear, la VM se descarga y **se provisiona una sola vez** (arranca, instala y se
apaga); después `shell start <name>` solo la enciende.

## Arquitectura

El sistema son **engranajes** aislados unidos por **puertos** y **adaptadores**,
con una única composición explícita en el binario `shell`.

```text
crates/
├── kernel/        # engranaje: dominio + casos de uso + puertos
├── host/          # utilidad: layout de rutas + fs
├── text/          # utilidad: plantillas
├── store/         # adaptador: ProfileStore   (SQLite · sqlx)
├── lima/          # adaptador: VmProvider + VmShell + BrowserBridge (limactl, ssh)
├── environment/   # adaptador: ShellEnvironment (dotfiles/env)
├── provision/     # adaptador: Provisioner (bootstrap del sistema)
└── app/           # binario: composición + CLI

templates/         # plantillas de texto (lima.yaml, env.sh, apply.sh, bootstrap.sh)
```

- **`kernel`** no conoce Lima, SQLite, el filesystem ni la terminal: solo su
  dominio, sus casos de uso y los puertos `ProfileStore`, `VmProvider`,
  `VmShell`, `ShellEnvironment`, `Provisioner` y `BrowserBridge`.
- Los adaptadores implementan esos puertos; el único lugar que los conoce a
  todos es `crates/app`.
- Configuración en **SQLite** (`~/.local/share/shell/shell.db`), no en TOML.
- Los **dotfiles** sí son archivos reales (`~/.local/share/shell/shells/<name>/home/`),
  materializados al home del guest en cada `enter`.
- El guest **solo** ve la carpeta montada, en `~/Workspace`; el home del host no se
  monta (se desactiva el mount por defecto de Lima) y `enter` arranca ahí.
- El mount usa **virtiofs** con `mountInotify` **desactivado**: la opción
  `mountInotify` de Lima rompe `mkdir`/`chmod` concurrentes sobre el share
  (EACCES intermitente) y hace fallar `npm install`. Ver
  [lima#2450](https://github.com/lima-vm/lima/issues/2450). Como contrapartida,
  los `inotify` del guest no ven cambios hechos en el host.
- El bootstrap instala **neovim** y el entorno del guest exporta `EDITOR=nvim`.
- El shell de login del guest es **zsh**: se instala como paquete y cada `enter`
  lo fija con `chsh`. Como `limactl shell` usa el `user.shell` de Lima (no
  `/etc/passwd`), `enter` le pasa `--shell` con el shell de login del guest.
- Al entrar, `shell` levanta un **puente de navegador**: un shim `xdg-open` en el
  guest reenvía las URLs al navegador del host por un túnel SSH inverso que vive
  mientras dure la sesión, así `gh auth login` y compañía abren el navegador de
  macOS.
- El **terminfo** del terminal del host se copia al guest en cada `enter`
  (`infocmp` en el host → `tic` en el guest), para que la edición de línea y
  `clear` funcionen con terminales como kitty o alacritty.
- Las **plantillas** de texto viven en `templates/` (raíz del repo); el binario
  las **copia** a `~/.local/share/shell/templates/` al arrancar y renderiza desde
  ahí (ruta de origen configurable con `SHELL_TEMPLATES`).
- `limactl` se localiza por `PATH` y, si falta, en ubicaciones típicas de
  Homebrew.

### Librerías

| Área | Crate |
|---|---|
| CLI | `clap` |
| Persistencia | `sqlx` (SQLite) |
| Modelo/serialización | `serde`, `serde_json` |
| Errores tipados | `thiserror` |
| Runtime async | `tokio` |
| Prompts | `dialoguer` |
| Tablas | `comfy-table` |
| Logs | `tracing`, `tracing-subscriber` |

## Layout en disco

```
~/.local/share/shell/shell.db               # configuración (SQLite)
~/.local/share/shell/lima/                  # LIMA_HOME aislado (VMs de `shell`)
~/.local/share/shell/templates/             # plantillas copiadas del repo `templates/`
~/.local/share/shell/machines/<name>.yaml   # lima.yaml generado por shell
~/.local/share/shell/shells/<name>/home/    # tus dotfiles por shell (editables)
```

## Requisitos

- macOS con Apple Silicon (driver `vz`) y Lima: `brew install lima`.
- El **primer `shell create` descarga la imagen base** (~600 MB), puede tardar.

## Desarrollo

```sh
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release     # binario en target/release/shell
```

Pruebas: reglas puras y casos de uso con adaptadores falsos (`kernel`) y
regresión del parseo de `limactl` (NDJSON y array) en `lima`.

## Estado

Primera versión funcional: `create` / `enter` / `remove` / `list` / `start` /
`stop`. Pendiente/ideas: sincronización en vivo de dotfiles, port-forwarding,
snapshots de shells, y TUI.
