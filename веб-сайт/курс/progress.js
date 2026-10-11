// Система прогресса курса
const PROGRESS_KEY = "zip-course-progress";

class ПрогрессКурса {
  constructor() {
    this.данные = this.загрузить();
  }

  загрузить() {
    try {
      const saved = localStorage.getItem(PROGRESS_KEY);
      return saved ? JSON.parse(saved) : {};
    } catch (e) {
      return {};
    }
  }

  сохранить() {
    localStorage.setItem(PROGRESS_KEY, JSON.stringify(this.данные));
  }

  отметить_урок(номер, статус) {
    const key = "урок_" + номер;
    const prev = this.данные[key];
    if (prev && prev.статус === "пройден" && статус === "начат") {
      this.обновить_интерфейс();
      return;
    }
    this.данные[key] = {
      статус: статус,
      время: new Date().toISOString(),
    };
    this.сохранить();
    this.обновить_интерфейс();
  }

  отметить_задачу(номер, статус) {
    this.данные["задача_" + номер] = {
      статус: статус,
      время: new Date().toISOString(),
    };
    this.сохранить();
    this.обновить_интерфейс();
  }

  пройдено_уроков() {
    return Object.keys(this.данные)
      .filter((k) => k.startsWith("урок_"))
      .filter((k) => this.данные[k].статус === "пройден").length;
  }

  всего_уроков() {
    return 8;
  }

  обновить_интерфейс() {
    const bar = document.querySelector(".progress-bar");
    if (bar) {
      const процент = (this.пройдено_уроков() / this.всего_уроков()) * 100;
      bar.style.width = Math.max(процент, 12) + "%";
      if (процент === 0) bar.style.width = "3.5rem";
      bar.textContent = this.пройдено_уроков() + " / " + this.всего_уроков();
    }

    document.querySelectorAll(".lesson-item").forEach((item) => {
      const номер = item.dataset.lesson;
      if (this.данные["урок_" + номер] && this.данные["урок_" + номер].статус === "пройден") {
        item.classList.add("пройден");
      }
    });
  }
}

const прогресс = new ПрогрессКурса();
document.addEventListener("DOMContentLoaded", () => прогресс.обновить_интерфейс());
window.прогресс = прогресс;
