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

        {/* Link used as a button */}
        <a href="#" onClick={() => saveDraft()}>
          Save draft
        </a>

        {/* Anchor with click handler, no href */}
        <a onClick={() => openDialog()}>Open dialog</a>

        {/* Clickable div */}
        <div onClick={() => setMenuOpen(!menuOpen)}>Menu</div>

        {/* Image missing alt text */}
        <img src="/hero.png" />

        {/* Unlabeled select */}
        <select>
          <option value="">Pick one</option>
        </select>

        {/* Unlabeled textarea */}
        <textarea placeholder="Enter a description" />

        {/* Nested JSX in conditional — should stay inside parent */}
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
