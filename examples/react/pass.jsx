import { useState } from "react";

export function SignupForm() {
  const [agreed, setAgreed] = useState(false);

  function handleSubmit(e) {
    e.preventDefault();
  }

  return (
    <main>
      <h1>Sign up</h1>

      <form onSubmit={handleSubmit}>
        {/* htmlFor label */}
        <label htmlFor="name">Full name</label>
        <input id="name" type="text" />

        {/* Wrapping label */}
        <label>
          Email address
          <input type="email" />
        </label>

        {/* Input button with value */}
        <input type="submit" value="Create account" />

        {/* Explicit button text */}
        <button type="button">Cancel</button>
      </form>

      {/* Navigation link */}
      <a href="/login">Already have an account?</a>

      {/* Image with alt text */}
      <img src="/logo.svg" alt="Acme Inc." />

      {/* Decorative image */}
      <img src="/border-flourish.png" alt="" />

      {/* Conditional rendering */}
      {agreed && (
        <p>
          Thanks for agreeing. <a href="/dashboard">Go to dashboard</a>
        </p>
      )}

      {/* Fragment with children */}
      <>
        <span>Status:</span>
        <span>{agreed ? "agreed" : "pending"}</span>
      </>
    </main>
  );
}
