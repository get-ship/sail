/**
 * Grades module — re-exports the store grades write path.
 * Same API for CLI and future viewer (HARNESS-2 `grade` module).
 */

export { writeGrades, type GradeInput } from '../store/grades.ts'
