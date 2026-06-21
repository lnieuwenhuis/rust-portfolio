(() => {
    const reduceMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;

    const revealItems = document.querySelectorAll("[data-reveal]");
    revealItems.forEach((item) => item.classList.add("is-visible"));

    if ("IntersectionObserver" in window && !reduceMotion) {
        const observer = new IntersectionObserver((entries) => {
            entries.forEach((entry) => {
                if (entry.isIntersecting) {
                    entry.target.classList.add("is-visible");
                    observer.unobserve(entry.target);
                }
            });
        }, { threshold: 0.16 });

        revealItems.forEach((item) => observer.observe(item));
    }

    if (!reduceMotion) {
        document.querySelectorAll(".magnetic-card").forEach((card) => {
            card.addEventListener("pointermove", (event) => {
                const rect = card.getBoundingClientRect();
                const x = (event.clientX - rect.left) / rect.width - 0.5;
                const y = (event.clientY - rect.top) / rect.height - 0.5;
                card.style.setProperty("--rx", `${(-y * 4).toFixed(2)}deg`);
                card.style.setProperty("--ry", `${(x * 5).toFixed(2)}deg`);
            });

            card.addEventListener("pointerleave", () => {
                card.style.removeProperty("--rx");
                card.style.removeProperty("--ry");
            });
        });

        const visual = document.querySelector("[data-orbit]");
        if (visual) {
            visual.addEventListener("pointermove", (event) => {
                const rect = visual.getBoundingClientRect();
                const x = (event.clientX - rect.left) / rect.width - 0.5;
                visual.style.setProperty("--visual-rotate", `${(x * 10).toFixed(2)}deg`);
            });
        }
    }

    document.querySelectorAll("[data-copy-email]").forEach((button) => {
        const original = button.textContent;
        button.addEventListener("click", async () => {
            const email = button.getAttribute("data-copy-email");
            try {
                await navigator.clipboard.writeText(email);
                button.textContent = "Copied";
                window.setTimeout(() => {
                    button.textContent = original;
                }, 1600);
            } catch {
                window.location.href = `mailto:${email}`;
            }
        });
    });
})();
