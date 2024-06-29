/*
  Warnings:

  - Added the required column `priority` to the `notifications` table without a default value. This is not possible if the table is not empty.

*/
-- CreateEnum
CREATE TYPE "Priority" AS ENUM ('HIGH', 'NORMAL', 'LOW');

-- AlterTable
ALTER TABLE "notifications" ADD COLUMN     "priority" "Priority" NOT NULL;
