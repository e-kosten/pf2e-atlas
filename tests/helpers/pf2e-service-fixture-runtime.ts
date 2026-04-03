export type ServiceTestFixture = {
  root: string;
  manifestPath: string;
};

export async function cleanupCreatedRoots(createdRoots: string[]): Promise<void> {
  await Promise.all(
    createdRoots.splice(0).map(async (root) => {
      await import("node:fs/promises").then(({ rm }) => rm(root, { recursive: true, force: true }));
    }),
  );
}
