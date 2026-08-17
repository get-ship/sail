/**
 * Round-trip type tests per studio surface (SPEC-STUDIO-CONTRACT-1 §3).
 *
 * Call generated methods with the exact arg shapes stores use; assert ret
 * shapes match store expectations. Wrong ret fields → tsc fails.
 * Deleting a METHOD_CONTRACTS method fails this file's typecheck.
 *
 * Included via tsconfig; exercised by typecheck-fail.test.ts / typecheck script.
 */
import type {
  CheckArgs,
  CheckRet,
  EditArgs,
  EditRet,
  InspectModelArgs,
  InspectModelRet,
  LayoutCheckArgs,
  ListFilesArgs,
  ListFilesRet,
  RenderScreenshotArgs,
  RenderScreenshotRet,
  StudioOpsMethods,
  TaskAddArtifactArgs,
  TaskGetArgs,
  TaskGetRet,
  TaskListArgs,
  TaskListRet,
  TaskThreadCreateArgs,
  TaskThreadListArgs,
  ThemeFontSearchArgs,
  ThemeFontSearchRet,
  ThemeForkArgs,
  ThemeListArgs,
  ThemeListRet,
  ThemeReadArgs,
  ThemeReadRet,
  ThemeWriteArgs,
  ThemeWriteRet,
} from '../src/index'

// --- Edit surface -----------------------------------------------------------

type EditClient = Pick<
  StudioOpsMethods,
  | 'sailInspectModel'
  | 'sailInspectTree'
  | 'sailInspectState'
  | 'sailRenderScreenshot'
  | 'sailAuthoringEdit'
  | 'sailAuthoringCheck'
  | 'sailAuthoringListFiles'
  | 'sailTaskList'
>

declare const edit: EditClient

async function editRoundTrip(): Promise<void> {
  const modelArgs: InspectModelArgs = { path: 'apps/studio/routes/page.sail' }
  const model: InspectModelRet = await edit.sailInspectModel(modelArgs)
  void model.file
  void model.modelVersion
  void model.tree
  void model.source

  const shotArgs: RenderScreenshotArgs = {
    path: 'apps/studio/routes/page.sail',
    theme: 'studio',
    mode: 'light',
  }
  const shot: RenderScreenshotRet = await edit.sailRenderScreenshot(shotArgs)
  void shot.imagePng
  void shot.regions
  void shot.ms

  // writeSource path — full-content check-guarded write via SailAuthoring/Edit
  const writeArgs: EditArgs = {
    path: 'apps/studio/routes/page.sail',
    ops: [],
    source: '<page></page>\n',
    writeSource: true,
    actor: 'human:studio',
  }
  const written: EditRet = await edit.sailAuthoringEdit(writeArgs)
  void written.applied
  void written.modelRevision
  void written.diagnostics

  const filesArgs: ListFilesArgs = { project: 'apps/studio' }
  const files: ListFilesRet = await edit.sailAuthoringListFiles(filesArgs)
  void files.files
  void files.ok

  const tasksArgs: TaskListArgs = { project: 'apps/studio', open: true }
  const tasks: TaskListRet = await edit.sailTaskList(tasksArgs)
  void tasks.tasks
  void tasks.ok
}

// --- Checks surface ---------------------------------------------------------

type ChecksClient = Pick<
  StudioOpsMethods,
  'sailAuthoringCheck' | 'sailAuthoringLayoutCheck' | 'sailInspectModel'
>

declare const checks: ChecksClient

async function checksRoundTrip(): Promise<void> {
  const args: CheckArgs = {
    path: 'apps/studio/routes/page.sail',
    fix: 'off',
  }
  const ret: CheckRet = await checks.sailAuthoringCheck(args)
  void ret.ok
  void ret.count
  void ret.diagnostics

  const layoutArgs: LayoutCheckArgs = { path: 'apps/studio/routes/page.sail' }
  const layout = await checks.sailAuthoringLayoutCheck(layoutArgs)
  void layout

  const model: InspectModelRet = await checks.sailInspectModel({
    path: 'apps/studio/routes/page.sail',
  })
  void model.source
}

// --- Themes / design surface ------------------------------------------------

type DesignClient = Pick<
  StudioOpsMethods,
  | 'sailThemeList'
  | 'sailThemeRead'
  | 'sailThemeWrite'
  | 'sailThemeFork'
  | 'sailThemeFontSearch'
  | 'sailThemeFontAdd'
  | 'sailInspectTokenUsage'
  | 'sailRenderScreenshot'
>

declare const design: DesignClient

async function themesRoundTrip(): Promise<void> {
  const listArgs: ThemeListArgs = { path: 'apps/studio/themes' }
  const list: ThemeListRet = await design.sailThemeList(listArgs)
  void list.files
  void list.ok

  const readArgs: ThemeReadArgs = { path: 'apps/studio/themes/studio.json' }
  const read: ThemeReadRet = await design.sailThemeRead(readArgs)
  void read.content

  const writeArgs: ThemeWriteArgs = {
    path: 'apps/studio/themes/studio.json',
    content: '{}',
  }
  const write: ThemeWriteRet = await design.sailThemeWrite(writeArgs)
  void write.bytes
  void write.ok

  const forkArgs: ThemeForkArgs = {
    source_path: 'packages/sail/tokens/model/themes/sail.json',
    dest_path: 'apps/studio/themes/fork.json',
    new_id: 'fork',
  }
  await design.sailThemeFork(forkArgs)

  // category + limit were consumed-unspecced — now contracted.
  const fontArgs: ThemeFontSearchArgs = {
    query: 'inter',
    category: 'sans-serif',
    limit: 20,
  }
  const fonts: ThemeFontSearchRet = await design.sailThemeFontSearch(fontArgs)
  void fonts.families
  void fonts.hits
  void fonts.catalogSize
  void fonts.source
  void fonts.ok

  await design.sailThemeFontAdd({ family: 'Inter' })
  await design.sailInspectTokenUsage({ token: 'color.bg' })
  await design.sailRenderScreenshot({
    source: '<page></page>',
    theme: 'studio',
    mode: 'dark',
  })
}

// --- Tasks / Work surface ---------------------------------------------------

type TasksClient = Pick<
  StudioOpsMethods,
  | 'sailTaskThreadList'
  | 'sailTaskList'
  | 'sailTaskGet'
  | 'sailTaskThreadCreate'
  | 'sailTaskAddArtifact'
  | 'sailTaskCommitIntent'
  | 'sailTaskReanchor'
  | 'sailTaskAgentNote'
>

declare const tasks: TasksClient

async function tasksRoundTrip(): Promise<void> {
  const listArgs: TaskThreadListArgs = {
    project: 'apps/studio',
    open: true,
    actor: 'human',
  }
  const list = await tasks.sailTaskThreadList(listArgs)
  void list.ok
  void list.tasks

  const fallback: TaskListRet = await tasks.sailTaskList({
    project: 'apps/studio',
  })
  void fallback.tasks

  const getArgs: TaskGetArgs = { project: 'apps/studio', id: 't1' }
  const got: TaskGetRet = await tasks.sailTaskGet(getArgs)
  void got.task
  void got.ok

  const createArgs: TaskThreadCreateArgs = {
    project: 'apps/studio',
    title: 'Review',
    body: 'looks good',
    anchor: {
      nodeId: 'n1',
      elementPath: 'page > card',
      contentHash: 'abc',
    },
    actor: { kind: 'human', name: 'user' },
  }
  await tasks.sailTaskThreadCreate(createArgs)

  // Batch tray attaches via AddArtifact (no shadow batch op).
  const artArgs: TaskAddArtifactArgs = {
    project: 'apps/studio',
    id: 't1',
    filename: 'crop-1.png',
    content_base64: 'AAAA',
    mime: 'image/png',
    kind: 'render',
    label: 'Selection',
    region: { nodeId: 'n1', kind: 'selection' },
    actor: { kind: 'human', name: 'user' },
  }
  await tasks.sailTaskAddArtifact(artArgs)

  await tasks.sailTaskCommitIntent({ project: 'apps/studio', id: 't1' })
  await tasks.sailTaskReanchor({
    project: 'apps/studio',
    id: 't1',
    anchor: { nodeId: 'n2', elementPath: 'x', contentHash: 'd' },
  })
  await tasks.sailTaskAgentNote({
    project: 'apps/studio',
    id: 't1',
    body: 'note',
  })
}

void editRoundTrip
void checksRoundTrip
void themesRoundTrip
void tasksRoundTrip
