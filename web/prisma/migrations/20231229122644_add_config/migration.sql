/*
  Warnings:

  - Added the required column `storeId` to the `users` table without a default value. This is not possible if the table is not empty.
  - Added the required column `storeName` to the `users` table without a default value. This is not possible if the table is not empty.

*/
-- AlterTable
ALTER TABLE "users" ADD COLUMN     "storeId" TEXT NOT NULL,
ADD COLUMN     "storeName" TEXT NOT NULL;
