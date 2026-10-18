import { smokePackagedWeb } from '../smoke-packaged-web.js';

const [binary, bundle, ...extra] = process.argv.slice(2);
if (!binary || !bundle || extra.length) {
  console.error('Usage: smoke-packaged-web <extracted-atlas-binary> <built-ui-directory>');
  process.exitCode = 2;
} else {
  await smokePackagedWeb(binary, bundle).then(() => {
    console.log('Packaged web HTML, JavaScript and CSS smoke test passed');
  }).catch((error: Error) => {
    console.error(error.message);
    process.exitCode = 1;
  });
}
