/*
  Warnings:

  - You are about to drop the column `storeId` on the `users` table. All the data in the column will be lost.
  - You are about to drop the column `storeName` on the `users` table. All the data in the column will be lost.
  - A unique constraint covering the columns `[configId]` on the table `users` will be added. If there are existing duplicate values, this will fail.
  - Added the required column `configId` to the `users` table without a default value. This is not possible if the table is not empty.

*/
-- AlterTable
ALTER TABLE "users" DROP COLUMN "storeId",
DROP COLUMN "storeName",
ADD COLUMN     "configId" TEXT NOT NULL;

-- CreateTable
CREATE TABLE "Config" (
    "id" TEXT NOT NULL,
    "storeName" TEXT NOT NULL,
    "storeId" TEXT NOT NULL,

    CONSTRAINT "Config_pkey" PRIMARY KEY ("id")
);

-- CreateIndex
CREATE UNIQUE INDEX "users_configId_key" ON "users"("configId");

-- AddForeignKey
ALTER TABLE "users" ADD CONSTRAINT "users_configId_fkey" FOREIGN KEY ("configId") REFERENCES "Config"("id") ON DELETE RESTRICT ON UPDATE CASCADE;
