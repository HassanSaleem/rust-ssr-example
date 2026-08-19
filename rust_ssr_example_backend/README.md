# Backend scripts

## `run-backend.sh`
Runs the backend (`cargo run`). Calls `setup-oracle-instantclient.sh` first,
then exports `LD_LIBRARY_PATH` so the Oracle driver can find its client
libraries.

```bash
./scripts/run-backend.sh
```

Extra arguments are forwarded to `cargo run`, e.g. `./scripts/run-backend.sh --release`.

Requires `rust_ssr_example_backend/.env` with `ORACLE_USERNAME`,
`ORACLE_PASSWORD` and `ORACLE_CONNECT_STRING` set (copy `.env.example`).

## `setup-oracle-instantclient.sh`
Idempotent installer for the Oracle Instant Client (Basic Light), required
by the `oracle` crate at runtime. No `sudo` needed for this part.

```bash
./scripts/setup-oracle-instantclient.sh
```

- Skips if `libclntsh.so` is already resolvable on the system, or already
  installed at `.instantclient/` (gitignored, downloaded on demand).
- Downloads and extracts the client into `.instantclient/` next to this
  script's parent directory.
- Also links a local `libaio.so.1` inside `.instantclient/` if the system
  only has `libaio.so.1t64` (Ubuntu's time64 SONAME transition), since
  Instant Client expects the old name.

If it warns that `libaio` is missing entirely, install it manually (needs root):

```bash
sudo apt install -y libaio1t64 || sudo apt install -y libaio1
```
