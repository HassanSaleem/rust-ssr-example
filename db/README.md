# DB scripts

## `run-liquibase.sh`
Runs Liquibase against the schema in `changelog/`. Downloads a portable
Liquibase distribution into `.liquibase/` (gitignored) on first use and
runs it directly, bypassing the outdated/confined `snap install liquibase`
package (its bundled JRE is Java 8 and can't load the `ojdbc11` driver,
and its confinement can't see the system JDK either).

```bash
JAVA_HOME=/usr/lib/jvm/java-21-openjdk-amd64 ./scripts/run-liquibase.sh update
```

Any Liquibase subcommand works, e.g. `status`, `rollback-count 1`, etc. —
all arguments are forwarded as-is.

Set `JAVA_HOME` to any installed JDK 11+ (check with `update-alternatives
--list java` or `ls /usr/lib/jvm`).

Requires:
- `liquibase.properties` in this directory with real credentials (copy
  `liquibase.properties.example`; the real file is gitignored).
- `lib/ojdbc11.jar` — download from Maven Central:
  ```bash
  mkdir -p lib
  curl -fL -o lib/ojdbc11.jar \
    https://repo1.maven.org/maven2/com/oracle/database/jdbc/ojdbc11/23.26.3.0.0/ojdbc11-23.26.3.0.0.jar
  ```
