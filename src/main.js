// ─── State ───────────────────────────────────────
let invoke;
let profiles = [];
let selectedColor = '#0068ff';
let editingProfileId = null;   // null = create mode, string = edit mode
let contextProfileId = null;

// ─── DOM Elements ────────────────────────────────
let grid, modalOverlay, modalTitle, nameInput, colorPicker;
let previewAvatar, previewName, saveBtn, closeBtn, cancelBtn;
let contextMenu, deleteOverlay, deleteNameEl, deleteCancelBtn, deleteConfirmBtn;

// ─── Init ────────────────────────────────────────
document.addEventListener('DOMContentLoaded', async () => {
  try {
    // Wait for Tauri IPC to be ready
    invoke = window.__TAURI__.core.invoke;

    // Cache DOM elements
    grid = document.getElementById('profiles-grid');
    modalOverlay = document.getElementById('modal-overlay');
    modalTitle = document.getElementById('modal-title');
    nameInput = document.getElementById('profile-name-input');
    colorPicker = document.getElementById('color-picker');
    previewAvatar = document.getElementById('preview-avatar');
    previewName = document.getElementById('preview-name');
    saveBtn = document.getElementById('modal-save-btn');
    closeBtn = document.getElementById('modal-close-btn');
    cancelBtn = document.getElementById('modal-cancel-btn');
    contextMenu = document.getElementById('context-menu');
    deleteOverlay = document.getElementById('delete-overlay');
    deleteNameEl = document.getElementById('delete-profile-name');
    deleteCancelBtn = document.getElementById('delete-cancel-btn');
    deleteConfirmBtn = document.getElementById('delete-confirm-btn');

    setupEventListeners();
    await loadProfiles();
  } catch (e) {
    console.error('Init error:', e);
    document.body.innerHTML = '<div style="color:white;padding:40px;font-family:sans-serif"><h2>Lỗi khởi tạo</h2><p>' + e + '</p></div>';
  }
});

// ─── Load & Render ───────────────────────────────
async function loadProfiles() {
  try {
    profiles = await invoke('list_profiles');
  } catch (e) {
    profiles = [];
    console.error('Failed to load profiles:', e);
  }
  renderGrid();
}

function renderGrid() {
  grid.innerHTML = '';

  // Profile cards
  profiles.forEach((p, i) => {
    const card = document.createElement('div');
    card.className = 'profile-card';
    card.style.setProperty('--card-accent', p.color);
    card.style.animationDelay = `${i * 0.05 + 0.05}s`;
    card.dataset.id = p.id;

    const initials = getInitials(p.name);

    card.innerHTML = `
      <button class="profile-more-btn" data-id="${p.id}" title="Tuỳ chọn">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
          <circle cx="12" cy="5" r="2"/><circle cx="12" cy="12" r="2"/><circle cx="12" cy="19" r="2"/>
        </svg>
      </button>
      <div class="profile-avatar" style="background:${p.color}">${initials}</div>
      <div class="profile-name">${escapeHtml(p.name)}</div>
    `;

    // Double-click or click to open
    card.addEventListener('dblclick', (e) => {
      if (e.target.closest('.profile-more-btn')) return;
      openProfile(p);
    });
    card.addEventListener('click', (e) => {
      if (e.target.closest('.profile-more-btn')) return;
      openProfile(p);
    });

    // Right-click context menu
    card.addEventListener('contextmenu', (e) => {
      e.preventDefault();
      showContextMenu(e, p);
    });

    // More button
    const moreBtn = card.querySelector('.profile-more-btn');
    moreBtn.addEventListener('click', (e) => {
      e.stopPropagation();
      const rect = moreBtn.getBoundingClientRect();
      showContextMenu({ clientX: rect.right, clientY: rect.bottom }, p);
    });

    grid.appendChild(card);
  });

  // Add card
  const addCard = document.createElement('div');
  addCard.className = 'add-card';
  addCard.style.animationDelay = `${profiles.length * 0.05 + 0.05}s`;
  addCard.innerHTML = `
    <div class="add-icon">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
        <line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>
      </svg>
    </div>
    <span class="add-label">Thêm Profile</span>
  `;
  addCard.addEventListener('click', () => openCreateModal());
  grid.appendChild(addCard);
}

// ─── Profile Actions ─────────────────────────────
async function openProfile(profile) {
  try {
    await invoke('open_profile', { id: profile.id, name: profile.name });
    showToast(`Đã mở ${profile.name}`);
  } catch (e) {
    showToast('Lỗi: ' + e);
  }
}

// ─── Modal ───────────────────────────────────────
function openCreateModal() {
  editingProfileId = null;
  modalTitle.textContent = 'Tạo Profile Mới';
  saveBtn.textContent = 'Tạo Profile';
  nameInput.value = '';
  selectColor('#25D366');
  updatePreview();
  modalOverlay.classList.remove('hidden');
  setTimeout(() => nameInput.focus(), 100);
}

function openEditColorModal(profile) {
  editingProfileId = profile.id;
  modalTitle.textContent = 'Đổi Màu Profile';
  saveBtn.textContent = 'Lưu';
  nameInput.value = profile.name;
  nameInput.disabled = true;
  selectColor(profile.color);
  updatePreview();
  modalOverlay.classList.remove('hidden');
}

function closeModal() {
  modalOverlay.classList.add('hidden');
  nameInput.disabled = false;
  editingProfileId = null;
}

async function handleSave() {
  const name = nameInput.value.trim();
  if (!name) {
    nameInput.focus();
    return;
  }

  try {
    if (editingProfileId) {
      // Update color
      await invoke('update_profile_color', { id: editingProfileId, color: selectedColor });
      showToast('Đã cập nhật profile');
    } else {
      // Create new
      await invoke('create_profile', { name, color: selectedColor });
      showToast(`Đã tạo "${name}"`);
    }
    closeModal();
    await loadProfiles();
  } catch (e) {
    showToast('Lỗi: ' + e);
  }
}

// ─── Color Picker ────────────────────────────────
function selectColor(color) {
  selectedColor = color;
  document.querySelectorAll('.color-swatch').forEach(s => {
    s.classList.toggle('selected', s.dataset.color === color);
  });
  updatePreview();
}

function updatePreview() {
  const name = nameInput.value.trim() || 'Tên profile';
  previewAvatar.textContent = getInitials(name);
  previewAvatar.style.background = selectedColor;
  previewName.textContent = name;
}

// ─── Context Menu ────────────────────────────────
function showContextMenu(e, profile) {
  contextProfileId = profile;
  contextMenu.classList.remove('hidden');

  const x = Math.min(e.clientX, window.innerWidth - 200);
  const y = Math.min(e.clientY, window.innerHeight - 220);
  contextMenu.style.left = x + 'px';
  contextMenu.style.top = y + 'px';
}

function hideContextMenu() {
  contextMenu.classList.add('hidden');
  contextProfileId = null;
}

// ─── Rename (inline) ─────────────────────────────
function startRename(profile) {
  hideContextMenu();
  const card = grid.querySelector(`[data-id="${profile.id}"]`);
  if (!card) return;

  const nameEl = card.querySelector('.profile-name');
  const input = document.createElement('input');
  input.type = 'text';
  input.className = 'inline-rename';
  input.value = profile.name;
  nameEl.replaceWith(input);
  input.focus();
  input.select();

  const finish = async () => {
    const newName = input.value.trim();
    if (newName && newName !== profile.name) {
      try {
        await invoke('rename_profile', { id: profile.id, newName });
        showToast(`Đổi tên thành "${newName}"`);
      } catch (e) {
        showToast('Lỗi: ' + e);
      }
    }
    await loadProfiles();
  };

  input.addEventListener('blur', finish);
  input.addEventListener('keydown', (e) => {
    if (e.key === 'Enter') input.blur();
    if (e.key === 'Escape') { input.value = profile.name; input.blur(); }
  });
}

// ─── Delete ──────────────────────────────────────
function showDeleteConfirm(profile) {
  hideContextMenu();
  contextProfileId = profile;
  deleteNameEl.textContent = profile.name;
  deleteOverlay.classList.remove('hidden');
}

async function confirmDelete() {
  if (!contextProfileId) return;
  try {
    await invoke('delete_profile', { id: contextProfileId.id });
    showToast(`Đã xoá "${contextProfileId.name}"`);
  } catch (e) {
    showToast('Lỗi: ' + e);
  }
  deleteOverlay.classList.add('hidden');
  contextProfileId = null;
  await loadProfiles();
}

// ─── Toast ───────────────────────────────────────
function showToast(msg) {
  let toast = document.querySelector('.toast');
  if (!toast) {
    toast = document.createElement('div');
    toast.className = 'toast';
    document.body.appendChild(toast);
  }
  toast.textContent = msg;
  toast.classList.remove('show');
  void toast.offsetWidth; // force reflow
  toast.classList.add('show');
  setTimeout(() => toast.classList.remove('show'), 2500);
}

// ─── Helpers ─────────────────────────────────────
function getInitials(name) {
  return name.split(/\s+/).map(w => w[0]).slice(0, 2).join('').toUpperCase() || '?';
}

function escapeHtml(str) {
  const d = document.createElement('div');
  d.textContent = str;
  return d.innerHTML;
}

// ─── Event Listeners ─────────────────────────────
function setupEventListeners() {
  // Modal
  closeBtn.addEventListener('click', closeModal);
  cancelBtn.addEventListener('click', closeModal);
  saveBtn.addEventListener('click', handleSave);
  nameInput.addEventListener('input', updatePreview);
  nameInput.addEventListener('keydown', (e) => { if (e.key === 'Enter') handleSave(); });
  modalOverlay.addEventListener('click', (e) => { if (e.target === modalOverlay) closeModal(); });

  // Color swatches
  colorPicker.addEventListener('click', (e) => {
    const swatch = e.target.closest('.color-swatch');
    if (swatch) selectColor(swatch.dataset.color);
  });

  // Context menu items
  document.getElementById('ctx-open').addEventListener('click', () => {
    if (contextProfileId) openProfile(contextProfileId);
    hideContextMenu();
  });
  document.getElementById('ctx-rename').addEventListener('click', () => {
    if (contextProfileId) startRename(contextProfileId);
  });
  document.getElementById('ctx-color').addEventListener('click', () => {
    if (contextProfileId) openEditColorModal(contextProfileId);
    hideContextMenu();
  });
  document.getElementById('ctx-delete').addEventListener('click', () => {
    if (contextProfileId) showDeleteConfirm(contextProfileId);
  });

  // Hide context menu on click outside
  document.addEventListener('click', (e) => {
    if (!e.target.closest('.context-menu') && !e.target.closest('.profile-more-btn')) {
      hideContextMenu();
    }
  });

  // Delete confirm
  deleteCancelBtn.addEventListener('click', () => { deleteOverlay.classList.add('hidden'); });
  deleteConfirmBtn.addEventListener('click', confirmDelete);
  deleteOverlay.addEventListener('click', (e) => { if (e.target === deleteOverlay) deleteOverlay.classList.add('hidden'); });
}
