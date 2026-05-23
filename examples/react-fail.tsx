export function App() {
  return (
    <main>
      <section>
        <h2>Inaccessible React Example</h2>
        <input type="text" />
        <div onClick={saveDraft}>Save draft</div>
        <a href="#" onClick={openDialog}>
          Open dialog
        </a>
        <button></button>
        <img src="/hero.png" />
      </section>
    </main>
  );
}
