import type { Metadata } from "next";
import { H1, P, DocsShell, H2, Code, Note, Ul } from "@/components/Sections";

export const metadata: Metadata = {
  title: "Installation",
  description:
    "Install ctx via npm, cargo, or prebuilt binaries. Requires Rust 1.85+ only when building from source.",
};

export default function InstallPage() {
  return (
    <DocsShell>
      <H1>Installation</H1>
      <P>
        ctx is a single binary. The quickest way to get it is npm, which avoids
        needing a Rust toolchain. Prebuilt binaries for macOS, Linux, and
        Windows are attached to every GitHub release.
      </P>

      <H2 id="requirements">System requirements</H2>
      <Ul>
        <li>
          <strong>Prebuilt binary / npm:</strong> macOS (Intel or Apple Silicon),
          Linux (x86_64 or aarch64), or Windows (x86_64 or arm64). No runtime
          dependencies.
        </li>
        <li>
          <strong>Building from source:</strong> Rust 1.85 or newer plus a C
          toolchain (only needed for the <Code>cargo install</Code> route).
        </li>
        <li>
          <strong>git:</strong> only <Code>ctx changed</Code> and{" "}
          <Code>ctx diff</Code> require a git repository. Everything else works
          on plain directories.
        </li>
      </Ul>

      <H2 id="npm">Via npm (recommended)</H2>
      <pre className="ctx-scroll overflow-x-auto rounded-lg border border-line bg-surface p-4 font-mono text-[13px] leading-6 text-ink">
{`npm install -g ctxai-cli
ctx --version`}
      </pre>
      <P>
        The npm package ships platform-specific binaries for macOS (x64 +
        arm64), Linux (x64 + arm64), and Windows (x64 + arm64). This is the same
        package the MCP examples use with{" "}
        <Code>npx -y ctxai-cli mcp</Code>.
      </P>

      <H2 id="cargo">Via cargo</H2>
      <pre className="ctx-scroll overflow-x-auto rounded-lg border border-line bg-surface p-4 font-mono text-[13px] leading-6 text-ink">
{`cargo install ctxai-cli --locked`}
      </pre>
      <P>
        Requires Rust 1.85 or newer. <Code>--locked</Code> pins the dependency
        graph to the versions ctx was tested against.
      </P>

      <H2 id="binaries">Prebuilt binaries</H2>
      <P>
        Download a binary from the{" "}
        <a
          href="https://github.com/halloffame12/CTX/releases"
          target="_blank"
          rel="noreferrer"
          className="font-semibold text-accent-deep hover:underline"
        >
          GitHub releases page
        </a>
        . Each release includes a <Code>checksums.txt</Code> so you can verify
        the download:
      </P>
      <pre className="ctx-scroll overflow-x-auto rounded-lg border border-line bg-surface p-4 font-mono text-[13px] leading-6 text-ink">
{`curl -LO https://github.com/halloffame12/CTX/releases/download/v0.1.2/ctx-linux-x86_64
shasum -a 256 ctx-linux-x86_64   # compare against checksums.txt
chmod +x ctx-linux-x86_64
sudo mv ctx-linux-x86_64 /usr/local/bin/ctx`}
      </pre>

      <Note>
        All installation methods produce the same <Code>ctx</Code> binary. There
        is no separate runtime, daemon, or service — the MCP server runs as a
        child process of your AI client.
      </Note>

      <H2 id="upgrade">Upgrading</H2>
      <P>
        npm users: <Code>npm install -g ctxai-cli@latest</Code>. Cargo users:{" "}
        <Code>cargo install ctxai-cli --locked --force</Code>. Check your
        version with <Code>ctx version</Code>.
      </P>

      <H2 id="smoke">Quick smoke test</H2>
      <pre className="ctx-scroll overflow-x-auto rounded-lg border border-line bg-surface p-4 font-mono text-[13px] leading-6 text-ink">
{`cd /path/to/your/project
ctx init
ctx doctor`}
      </pre>
      <P>
        <Code>ctx doctor</Code> verifies the index and reports anything out of
        date.
      </P>
    </DocsShell>
  );
}