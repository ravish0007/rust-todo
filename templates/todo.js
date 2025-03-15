let currentEditingElement = null;

const toggleNoteEdit = (noteElement) => {
  if (!noteElement) {
    return;
  }

  const content = noteElement.querySelector(".content");
  const contentInput = noteElement.querySelector(".content-input");
  const updateButton = noteElement.querySelector(".update-btn");
  const saveButton = noteElement.querySelector(".save-btn");

  content.classList.toggle("hidden");
  contentInput.classList.toggle("hidden");
  updateButton.classList.toggle("hidden");
  saveButton.classList.toggle("hidden");

  if (!contentInput.classList.contains("hidden")) {
    contentInput.focus();
    const length = contentInput.value.length;
    contentInput.setSelectionRange(length, length); // Move cursor to the end
  }
};

document.addEventListener("DOMContentLoaded", function () {
  document.querySelectorAll(".delete-btn").forEach((button) => {
    button.addEventListener("click", function () {
      const todoId = this.dataset.id;
      sendFormData(`/todos`, "DELETE", { id: parseInt(todoId) }).then(() => {});
    });
  });

  document.querySelectorAll(".update-btn").forEach((content) => {
    content.addEventListener("click", function (event) {
      event.stopPropagation();
      toggleNoteEdit(currentEditingElement);
      currentEditingElement = event.target.closest(".note-root");
      toggleNoteEdit(event.target.closest(".note-root"));
    });
  });

  document.querySelectorAll(".save-btn").forEach((content) => {
    content.addEventListener("click", function (event) {
      event.stopPropagation();

      rootNode = event.target.closest(".note-root");
      const dataset = rootNode.dataset;

      const todoId = dataset.id;
      const isDone = dataset.done;
      const content = rootNode.querySelector(".content-input").value;

      sendFormData(`/todos`, "PUT", {
        id: parseInt(todoId),
        content: content,
        done: isDone == "true" ? true : false,
      }).then(() => {});
    });
  });

  document.querySelectorAll(".rounded-checkbox").forEach((button) => {
    button.addEventListener("click", function (event) {
      event.stopPropagation();

      const dataset = event.currentTarget.closest(".note-root").dataset;

      const todoId = dataset.id;
      const isDone = dataset.done;
      const content = dataset.content;

      sendFormData(`/todos`, "PUT", {
        id: parseInt(todoId),
        content: content,
        done: isDone == "true" ? false : true,
      }).then(() => {});
    });
  });

  document.querySelectorAll(".content").forEach((content) => {
    content.addEventListener("dblclick", function (event) {
      event.stopPropagation();
      toggleNoteEdit(event.currentTarget.closest(".note-root"));
    });
  });

  document.querySelectorAll(".content-input").forEach((content) => {
    content.addEventListener("keypress", function (event) {
      event.stopPropagation();

      if (event.key === "Enter" && event.target.value.trim() != "") {
        const dataset = event.target.closest(".note-root").dataset;

        const todoId = dataset.id;
        const isDone = dataset.done;
        const content = event.target.value;

        sendFormData(`/todos`, "PUT", {
          id: parseInt(todoId),
          content: content,
          done: isDone == "true" ? true : false,
        }).then(() => {});
      }
    });
  });

  // document.querySelectorAll(".").forEach((content) => {
  //   content.addEventListener("blur", function (event) {
  //     event.stopPropagation();
  //     toggleNoteEdit(event.target.closest(".note-root"));
  //   });
  // });

  document
    .getElementById("newcontent")
    .addEventListener("keypress", function (event) {
      if (event.key === "Enter") {
        event.preventDefault(); // Prevent default form submission

        if (event.target.value.trim() != "") {
          document.getElementById("newtodo-form").submit(); // Submit the form
        }
      }
    });
});
