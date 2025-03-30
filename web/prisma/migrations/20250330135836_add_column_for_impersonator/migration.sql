-- AlterEnum
ALTER TYPE "NotificationType" ADD VALUE 'RATELIMIT_REACHED';

-- AlterTable
ALTER TABLE "sessions" ADD COLUMN     "impersonatorUserId" TEXT;

-- AddForeignKey
ALTER TABLE "sessions" ADD CONSTRAINT "sessions_impersonatorUserId_fkey" FOREIGN KEY ("impersonatorUserId") REFERENCES "users"("id") ON DELETE SET NULL ON UPDATE CASCADE;
