export function AccessiblePage() {
  return (
    <main>
      {/* Button with visible text — recognized via resolve_element */}
      <Button onClick={() => {}}>Save draft</Button>

      {/* Link with href + text — recognized as <a> */}
      <Link to="/reports">View reports</Link>

      {/* Image with alt text — recognized as <img> */}
      <Image src="/hero.png" alt="Hero banner" />
      <Image src="/border.png" alt="" />

      {/* FormLabel + TextField — recognized as <label> + <input> */}
      <FormLabel htmlFor="email">Email</FormLabel>
      <TextField id="email" />

      {/* Wrapping label with input — native elements, no mapping needed */}
      <label>
        Full name
        <input type="text" />
      </label>

      {/* IconButton with aria-label on the rendered button */}
      <button aria-label="Close dialog">
        <span aria-hidden="true">X</span>
      </button>
    </main>
  );
}
