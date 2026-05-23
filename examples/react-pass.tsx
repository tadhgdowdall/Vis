export function App() {
  return (
    <main>
      <section>
        <h2>Accessible React Example</h2>
        <label htmlFor="search">Search</label>
        <input id="search" type="search" />
        <button type="button">Open filters</button>
        <a href="/reports">View reports</a>
        <img src="/chart.png" alt="Quarterly revenue increased by 12 percent" />
      </section>
    </main>
  );
}
