-- CreateTable
CREATE TABLE "notifications" (
    "id" TEXT NOT NULL,
    "content" TEXT NOT NULL,
    "context" JSONB NOT NULL,
    "read" BOOLEAN NOT NULL DEFAULT false,

    CONSTRAINT "notifications_pkey" PRIMARY KEY ("id")
);
