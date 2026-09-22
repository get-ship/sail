/**
 * bundle.ts — tree-shake harness for the compile-away bake-off.
 *
 * Bundles ONE compiled render (render/<fixture>.js) plus its runtime import
 * graph with esbuild (--bundle --minify --tree-shaking), then measures
 * shipped bytes against the @ship/sail-runtime denominator.
 *
 * Usage:
 *   bun tests/bakeoff/shake/bundle.ts                 # both fixtures
 *   bun tests/bakeoff/shake/bundle.ts counter         # one fixture
 *
 * Outputs per fixture under dist/<fixture>/: main.js, main.js.gz,
 * esbuild-meta.json (surviving modules), summary.json.
 *
 * No dependencies are added to any package.json: esbuild arrives via
 * `bunx esbuild` (CLI), and `@ship/sail-runtime/*` specifiers resolve to the
 * runtime's TS sources through esbuild's --alias flag (mirroring the
 * runtime package.json "bun"/"development" export conditions).
 */
import { mkdir } from "node:fs/promises";

const WORKTREE = `${import.meta.dir}/../../..`;
const RUNTIME_SRC = `${WORKTREE}/packages/sail/runtime/src`;
const RENDER_DIR = `${import.meta.dir}/render`;
const OUT_ROOT = `${import.meta.dir}/dist`;

interface EsbuildMetafile {
  inputs: Record<string, { bytes: number }>;
  outputs: Record<
    string,
    { bytes: number; inputs: Record<string, { bytesInOutput: number }> }
  >;
}

async function bundle(fixture: string): Promise<void> {
  const entry = `${RENDER_DIR}/${fixture}.js`;
  const outdir = `${OUT_ROOT}/${fixture}`;
  const outfile = `${outdir}/main.js`;
  await mkdir(outdir, { recursive: true });

  // Bare workspace specifiers resolve like the runtime package.json "bun"
  // condition does: subpath -> TS source, no node_modules needed.
  const alias = [
    `@ship/sail-runtime=${RUNTIME_SRC}`,
    `@ship/tokens=${WORKTREE}/packages/sail/tokens/src`,
  ];
  const proc = Bun.spawnSync({
    cmd: [
      "bunx",
      "esbuild",
      entry,
      "--bundle",
      "--minify",
      "--tree-shaking=true",
      "--format=esm",
      "--platform=browser",
      "--target=es2022",
      ...alias.map((a) => `--alias:${a}`),
      `--outfile=${outfile}`,
      `--metafile=${outdir}/esbuild-meta.json`,
      "--log-level=warning",
    ],
    cwd: WORKTREE,
    stdout: "pipe",
    stderr: "pipe",
  });
  if (proc.exitCode !== 0) {
    console.error(`[bundle] esbuild failed for ${fixture}:\n${proc.stderr.toString()}`);
    process.exit(1);
  }

  const rawBuf = await Bun.file(outfile).arrayBuffer();
  const gzBuf = Bun.gzipSync(new Uint8Array(rawBuf), { level: 9 });
  await Bun.write(`${outfile}.gz`, gzBuf);
  const meta = (await Bun.file(`${outdir}/esbuild-meta.json`).json()) as EsbuildMetafile;
  const output = Object.values(meta.outputs)[0]!;

  const modules = Object.entries(output.inputs)
    .map(([p, info]) => ({
      path: p,
      bytes: info.bytesInOutput,
      runtime:
        p.includes("/packages/sail/runtime/src/") ||
        p.startsWith("packages/sail/runtime/src/"),
    }))
    .sort((a, b) => b.bytes - a.bytes);

  let runtimeBytes = 0;
  let renderBytes = 0;
  for (const m of modules) {
    if (m.runtime) runtimeBytes += m.bytes;
    else renderBytes += m.bytes;
  }

  const summary = {
    fixture,
    entry,
    command:
      "bunx esbuild <entry> --bundle --minify --tree-shaking=true --format=esm --platform=browser --target=es2022 --alias:@ship/sail-runtime=packages/sail/runtime/src --metafile=<dir>/esbuild-meta.json",
    bytesRaw: rawBuf.byteLength,
    bytesGzip: gzBuf.byteLength,
    moduleCount: modules.length,
    runtimeModuleCount: modules.filter((m) => m.runtime).length,
    runtimeBytesInOutput: runtimeBytes,
    renderAndControllerBytesInOutput: renderBytes,
    modules,
  };
  await Bun.write(`${outdir}/summary.json`, `${JSON.stringify(summary, null, 2)}\n`);

  console.log(
    `[bundle] ${fixture}: raw=${summary.bytesRaw} gzip=${summary.bytesGzip} modules=${summary.moduleCount} (runtime ${summary.runtimeModuleCount}, runtime-bytes=${runtimeBytes})`,
  );
}

const arg = process.argv[2];
const fixtures = arg ? [arg] : ["counter", "list"];
for (const f of fixtures) {
  if (f !== "counter" && f !== "list") throw new Error(`unknown fixture: ${f}`);
}
for (const f of fixtures) await bundle(f);
