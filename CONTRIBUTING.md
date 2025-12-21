# **Contributing to `rs_events`**

Thanks for your interest in contributing to **`rs_events`**! 🎉
This crate is an event-driven library for Rust, and community involvement is always welcome.

Feel free to explore the codebase, projects, or examples. If you have questions or need clarification, just ask — I’ll do my best to respond quickly.

✨ **Important:** Be positive and respectful. Constructive criticism is great — negativity is not. Let’s keep this a friendly and collaborative space.

---

## **How to Contribute**

### 🐞 Report Bugs

Found a bug? Open an [issue](../../issues) with a clear description, steps to reproduce, and expected behavior.

### 💡 Propose Features

Have an idea? Start a discussion by opening an [issue](../../issues) describing the feature and its use case.

### 🔧 Submit Fixes

Ready to code? Fork the repo, make your changes, and open a PR. See the step-by-step guide below.


---

## **Step-by-Step Guide**

**1.** Fork the repository
[How to fork a repo](https://docs.github.com/en/get-started/quickstart/fork-a-repo)

**2.** Clone your fork locally:

```sh
git clone <your-fork-url>
```

**3.** Move into the project directory:

```sh
cd rs_events
```

**4.** Add the original repo as `upstream`:

```sh
git remote add upstream https://github.com/GreenJ84/rs_events.git
```

**5.** Verify your remotes:

```sh
git remote -v
```

**6.** Sync with upstream to stay current:

```sh
git pull upstream main
```

**7.** Create a new feature branch:

```sh
git checkout -b feature/<your-branch-name>
```

**8.** Stage changes:

```sh
git add .
```

**9.** Make your modifications or improvements.

**10.** Commit changes:

```sh
git commit -m "feat: <short description of change>"
```

**11.** Push your branch:

```sh
git push -u origin feature/<your-branch-name>
```

**12.** Open a Pull Request (PR)

* Compare your branch with the `main` branch of this repository.
* Give your PR a clear **title** and **description**.
* Add screenshots or examples if relevant.
* Explain *why* the change is useful.

**13.** Submit the PR — PR Checklist 🚀

Before submitting, please ensure:

* ✅ Code is formatted with `cargo fmt`
* ✅ Lints pass with `cargo clippy`
* ✅ Tests pass with `cargo test`

---

### 🙏 Thanks for contributing

Your work makes `rs_events` better for everyone.

Credit to [Sanket1308](https://github.com/Sanket1308) for the original contributing template inspiration.
