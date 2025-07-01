window.addEventListener('keydown', (e) => {
  if (['ArrowDown', 'ArrowUp'].includes(e.key)) {
    e.preventDefault(); // Stop default smooth scroll
    const scrollAmount = window.innerHeight;
    const direction = e.key === 'ArrowDown' ? 1 : -1;
    window.scrollBy({ top: scrollAmount * direction, behavior: 'instant' });
  }
});

const observer = new IntersectionObserver((entries) => {
  entries.forEach(entry => {
    if (entry.isIntersecting) {
      history.replaceState(null, '', `#${entry.target.id}`);
      console.clear();
      console.log(`%c#${entry.target.id}: ${entry.target.dataset.notes}`, "font-family: 'Inter', system-ui; font-size: 20px");
    }
  });
}, {
  threshold: 0.5
});

document.querySelectorAll('.slide').forEach(section => {
  observer.observe(section);
});
