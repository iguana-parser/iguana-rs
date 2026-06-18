// CSS files are imported for their side effects; the bundler injects them. This
// ambient declaration lets the package type-check on its own, the way the
// consuming Vite apps already resolve these imports.
declare module "*.css";
