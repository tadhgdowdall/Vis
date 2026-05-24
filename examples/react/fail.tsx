import { useState } from "react";

export function InaccessibleSignup() {
  const [menuOpen, setMenuOpen] = useState(false);

  return (
    <main>
      <h1>Sign up</h1>

      <form>
        {/* Unlabeled text input */}
        <input type="text" />

        {/* Empty button */}
        <button></button>

        {/* Link used as a button — no real href */}
        <a href="#" onClick={() => saveDraft()}>
          Save draft
        </a>

        {/* Anchor with click handler but no href */}
        <a onClick={() => openDialog()}>Open dialog</a>

        {/* Clickable div with no semantics */}
        <div onClick={() => setMenuOpen(!menuOpen)}>Menu</div>

        {/* Image missing alt text */}
        <img src="/hero.png" />

        {/* Unlabeled select */}
        <select>
          <option value="">Pick one</option>
        </select>

        {/* Unlabeled textarea */}
        <textarea placeholder="Enter a description" />

        {/* Conditional JSX — nested inside expression container */}
        {menuOpen && (
          <div>
            <a href="/settings">Settings</a>
            <button>Log out</button>
          </div>
        )}
      </form>
    </main>
  );
}
