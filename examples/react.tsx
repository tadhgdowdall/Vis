export function App() {
  return (
    <main>
      <img src="/hero.png" />
      <div onClick={saveDraft}>Save draft</div>
      <button aria-label="Close" />
    </main>
  );
}
