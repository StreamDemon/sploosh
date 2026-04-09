# Runbook: New Project Setup

> Step-by-step guide to creating a new Sploosh project from scratch.

## Steps

1. **Create the project:**
   ```bash
   sploosh new my-project
   cd my-project
   ```

2. **Review the generated structure:**
   ```
   my-project/
   ├── sploosh.toml
   └── src/
       └── main.sp
   ```

3. **Edit `sploosh.toml`** to add dependencies and configure targets.

4. **Write your entry point** in `src/main.sp`:
   ```sploosh
   fn main() -> Result<(), AppError> {
       print("Hello, Sploosh!");
       Ok(())
   }
   ```

5. **Build and run:**
   ```bash
   sploosh run
   ```

6. **Initialize git:**
   ```bash
   git init
   git add .
   git commit -m "Initial project setup"
   ```

<!-- TODO: Expand with common project templates (web API, CLI tool, smart contract) once available -->
