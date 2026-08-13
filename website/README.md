# ctx — website

Marketing / documentation site for **ctx**, the codebase intelligence and
context engine for AI coding agents.

- **Stack**: Next.js 16 (App Router), React 19, Tailwind CSS v4, TypeScript.
- **Output**: fully static export (`next.config.ts` sets `output: "export"`) served
  from the custom domain
  [https://ctx.sumitchauhan.me/](https://ctx.sumitchauhan.me/).
- **Deploy**: automatic via `.github/workflows/website.yml` (GitHub Actions →
  Pages) on any push touching `website/**`.

## Develop

```bash
npm install
npm run dev       # local dev server
npm run build     # static export into out/
npm run lint      # eslint
```