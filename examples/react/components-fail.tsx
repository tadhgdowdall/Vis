export function InaccessiblePage() {
  return (
    <main>
      {/* Button without accessible name — recognized as button */}
      <Button onClick={() => {}} />

      {/* Another empty button */}
      <Button onClick={() => {}} />

      {/* Link without text — recognized as <a> */}
      <NavLink to="/reports" />

      {/* Image without alt — recognized as <img> */}
      <Image src="/hero.png" />

      {/* Form controls without labels — recognized by resolve_element */}
      <TextField />
      <Dropdown />
      <RichText />

      {/* Link without text */}
      <HyperLink to="/shop" />

      {/* Image component without alt */}
      <Avatar src="/user.png" />
    </main>
  );
}
