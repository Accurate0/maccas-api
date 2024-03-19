/*
  Warnings:

  - Changed the column `role` on the `users` table from a scalar field to a list field. If there are non-null values in that column, this step will fail.

*/
-- AlterTable
ALTER TABLE "users"
ALTER COLUMN "role" DROP DEFAULT,
ALTER COLUMN "role" SET DATA TYPE "Role"[] USING ARRAY["role"],
ALTER COLUMN "role" SET DEFAULT ARRAY['USER']::"Role"[];
