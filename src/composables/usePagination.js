import { ref, computed } from "vue";

// 分页功能（当前为 pageSize = 20）

export function usePagination(items, pageSize = 20) {
  const currentPage = ref(1);

  const paginatedItems = computed(() => {
    const list = items.value || [];
    const start = (currentPage.value - 1) * pageSize;
    return list.slice(start, start + pageSize);
  });

  const totalPages = computed(() => {
    const len = (items.value || []).length;
    return Math.ceil(len / pageSize) || 1;
  });

  function handlePageChange(page) {
    currentPage.value = page;
    window.scrollTo({ top: 0, behavior: "smooth" });
  }

  function resetPage() {
    currentPage.value = 1;
  }

  return {
    currentPage,
    paginatedItems,
    totalPages,
    handlePageChange,
    resetPage,
    pageSize,
  };
}
