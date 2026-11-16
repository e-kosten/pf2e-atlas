# Exact chunk-selection heap patch

This historical patch targets `sqlite-vec.c` from the published sqlite-vec 0.1.9 Cargo package. Its target and both hashes refer to the exact source used in the [research experiment](README.md). For research in a disposable checkout, run the extraction command from the Atlas checkout root. The second filter restores the unified diff's space marker for a blank context line.

```sh
sed -n '/^```diff$/,/^```$/p' docs/research/semantic-search-heap/exact-heap-patch.md |
  sed '1d;$d;s/^$/ /' > exact-heap.patch
```

| File | SHA-256 |
|---|---|
| Original `sqlite-vec.c` | `ba081a47fa02eadc3cf6b16c314b695b84081269349aac722b4efa338fe8fd85` |
| Extracted patch | `6a73d34ffaa665fc3de3ce941c0fa7995b7108907fcc9fbc74a6876267371541` |
| Patched `sqlite-vec.c` | `55d30b3ea8c56ef7c66231db2a38a69010ec4aea873b9e69246477a8df4894f0` |

Verify the original hash before applying the extracted patch to a disposable copy of the pinned C source. Change into the disposable source directory outside the Atlas checkout. Pass the path to the extracted patch to `git apply --check` and `git apply`, then verify the patched C hash.

```diff
--- a/sqlite-vec.c
+++ b/sqlite-vec.c
@@ -5954,7 +5954,7 @@
  * @param k: Size of output array
  * @return int
  */
-int min_idx(const f32 *distances, i32 n, u8 *candidates, i32 *out, i32 k,
+static int min_idx_legacy(const f32 *distances, i32 n, u8 *candidates, i32 *out, i32 k,
             u8 *bTaken, i32 *k_used) {
   assert(k > 0);
   assert(k <= n);
@@ -5983,6 +5983,82 @@
     bitmap_set(bTaken, min_idx, 1);
   }
   *k_used = k;
+  return SQLITE_OK;
+}
+
+/* Exact finite chunk ordering: smaller distance, then later physical index. */
+static int min_idx_better(const f32 *distances, i32 a, i32 b) {
+  return distances[a] < distances[b] ||
+         (distances[a] == distances[b] && a > b);
+}
+
+/* Worst-first heap; leaf test precedes the index multiplication. */
+static void min_idx_sift(const f32 *distances, i32 *heap, i32 length,
+                         i32 root) {
+  i32 value = heap[root];
+  while (root < length / 2) {
+    i32 child = root * 2 + 1;
+    if (child + 1 < length &&
+        min_idx_better(distances, heap[child], heap[child + 1])) {
+      child++;
+    }
+    if (!min_idx_better(distances, value, heap[child])) {
+      break;
+    }
+    heap[root] = heap[child];
+    root = child;
+  }
+  heap[root] = value;
+}
+
+int min_idx(const f32 *distances, i32 n, u8 *candidates, i32 *out, i32 k,
+            u8 *bTaken, i32 *k_used) {
+  assert(k > 0);
+  assert(k <= n);
+
+  /* Classify IEEE-f32 bytes without evaluating even masked-out signaling NaNs. */
+  for (i32 i = 0; i < n; i++) {
+    u32 encoding;
+    memcpy(&encoding, distances + i, sizeof(encoding));
+    if ((encoding & 0x7f800000u) == 0x7f800000u) {
+      return min_idx_legacy(distances, n, candidates, out, k, bTaken, k_used);
+    }
+  }
+
+  bitmap_clear(bTaken, n);
+  i32 used = 0;
+  for (i32 i = 0; i < n; i++) {
+    if (!bitmap_get(candidates, i)) {
+      continue;
+    }
+    if (used < k) {
+      i32 position = used++;
+      while (position > 0) {
+        i32 parent = (position - 1) / 2;
+        if (!min_idx_better(distances, out[parent], i)) {
+          break;
+        }
+        out[position] = out[parent];
+        position = parent;
+      }
+      out[position] = i;
+    } else if (min_idx_better(distances, i, out[0])) {
+      out[0] = i;
+      min_idx_sift(distances, out, used, 0);
+    }
+  }
+
+  /* In-place heapsort yields exactly distance-ascending/index-descending order. */
+  for (i32 end = used - 1; end > 0; end--) {
+    i32 swap = out[end];
+    out[end] = out[0];
+    out[0] = swap;
+    min_idx_sift(distances, out, end, 0);
+  }
+  for (i32 i = 0; i < used; i++) {
+    bitmap_set(bTaken, out[i], 1);
+  }
+  *k_used = used;
   return SQLITE_OK;
 }

```
