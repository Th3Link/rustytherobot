const $ = (id) => document.getElementById(id);
const robots = new Map();
const availableNames = new Set();
const colors = ['#bef46a', '#73c7ff', '#c5a0ff', '#ffb36c', '#ff87ad', '#65dfc4'];
const icon = '<svg viewBox="0 0 32 32" fill="none" aria-hidden="true"><path d="M16 4v4M14 4h4M6 14H3v9h3m20-9h3v9h-3M11 27v3m10-3v3" stroke="currentColor" stroke-width="2" stroke-linecap="round"/><rect x="6" y="9" width="20" height="18" rx="5" fill="currentColor" fill-opacity=".12" stroke="currentColor" stroke-width="2"/><rect x="10" y="14" width="4" height="4" rx="1" fill="currentColor"/><rect x="18" y="14" width="4" height="4" rx="1" fill="currentColor"/><path d="M12 22h8" stroke="currentColor" stroke-width="2" stroke-linecap="round"/></svg>';
let selected = null, cell = 52, center = { x: 0, y: 0 }, busy = false, dialogMode = 'add', pendingRename = null;
let toastTimer;
let board = null, chargepads = [], boardFit = true;
function fitBoard() {
  if (!board) return;
  boardFit = true;
  cell = Math.max(2, Math.min(64, ($('arena').clientWidth - 64) / board.x, ($('arena').clientHeight - 64) / board.y));
  center = { x: -Math.floor(board.x / 2) + (board.x - 1) / 2, y: -Math.floor(board.y / 2) + (board.y - 1) / 2 };
  renderArena();
}
let known = [];
try { known = JSON.parse(localStorage.getItem('rusty-names') || '[]'); } catch { /* Storage may be unavailable. */ }
const color = (team) => colors[((team % colors.length) + colors.length) % colors.length];
function remember() { try { localStorage.setItem('rusty-names', JSON.stringify([...robots.keys()])); } catch {} }
function toast(message, error = false) {
  $('toast').textContent = message; $('toast').classList.toggle('error', error); $('toast').hidden = false;
  clearTimeout(toastTimer); toastTimer = setTimeout(() => $('toast').hidden = true, 5500);
}
function log(message, team) {
  if (!$('activity').querySelector('time')) $('activity').replaceChildren();
  const row = document.createElement('li');
  const time = document.createElement('time'); time.textContent = new Date().toLocaleTimeString('de-DE');
  const dot = document.createElement('i'); dot.className = 'event-dot'; if (team !== undefined) dot.style.setProperty('--team', color(team));
  const text = document.createElement('span'); text.textContent = message;
  row.append(time, dot, text); $('activity').prepend(row);
  while ($('activity').children.length > 35) $('activity').lastChild.remove();
}
async function request(path, method, body) {
  const response = await fetch(`/${path}`, { method, headers: { 'Content-Type': 'application/json' }, ...(body === undefined ? {} : { body: JSON.stringify(body) }), signal: AbortSignal.timeout(8000) });
  if (!response.ok) throw new Error(`API-Fehler ${response.status}: ${path}`);
  return response.status === 204 ? null : response.text().then(text => text ? JSON.parse(text) : null);
}
function validState(state) { return state && ['x', 'y', 'health', 'battery', 'team_id'].every(key => Number.isInteger(state[key])); }
function update(name, state) {
  if (typeof name !== 'string' || !validState(state)) throw new Error('Ungültiger Roboterzustand');
  availableNames.add(name); robots.set(name, { ...state }); if (!selected) selected = name; remember(); render();
}
async function load(name) { const state = await request('robot_state', 'POST', { name }); update(name, state); }
function select(name) { selected = name; render(); }
function render() { renderArena(); renderControls(); renderTeams(); $('robot-count').textContent = robots.size; }
function renderArena() {
  const arena = $('arena'), grid = $('grid'), width = arena.clientWidth, height = arena.clientHeight;
  grid.replaceChildren();
  const originX = width / 2 - center.x * cell, originY = height / 2 + center.y * cell;
  grid.style.backgroundSize = `${cell}px ${cell}px`;
  grid.style.backgroundPosition = `${originX - cell / 2}px ${originY - cell / 2}px`;
  if (board) {
    const boundary = document.createElement('div'); boundary.className = 'board-boundary';
    const minX = -Math.floor(board.x / 2), maxY = -Math.floor(board.y / 2) + board.y - 1;
    boundary.style.left = `${originX + (minX - .5) * cell}px`;
    boundary.style.top = `${originY - (maxY + .5) * cell}px`;
    boundary.style.width = `${board.x * cell}px`; boundary.style.height = `${board.y * cell}px`;
    boundary.setAttribute('role', 'img'); boundary.setAttribute('aria-label', `Spielfeld: ${board.x} × ${board.y} Felder`);
    grid.append(boundary);
  }
  for (const pad of chargepads) {
    const tile = document.createElement('div'); tile.className = 'chargepad'; tile.textContent = 'ϟ';
    tile.style.left = `${originX + (pad.x - .5) * cell + 3}px`; tile.style.top = `${originY - (pad.y + .5) * cell + 3}px`;
    tile.style.width = tile.style.height = `${Math.max(1, cell - 6)}px`;
    tile.style.fontSize = `${Math.min(27, cell * .5)}px`;
    tile.title = `Ladefeld (${pad.x}, ${pad.y})`;
    tile.setAttribute('role', 'img'); tile.setAttribute('aria-label', tile.title); grid.append(tile);
  }
  $('empty-arena').hidden = robots.size > 0;
  for (let x = Math.ceil(center.x - width / cell / 2); x <= center.x + width / cell / 2; x++) {
    const label = document.createElement('span'); label.className = 'coordinate'; label.textContent = x; label.style.left = `${originX + x * cell - 3}px`; label.style.top = '8px'; grid.append(label);
  }
  for (let y = Math.ceil(center.y - height / cell / 2); y <= center.y + height / cell / 2; y++) {
    const label = document.createElement('span'); label.className = 'coordinate'; label.textContent = y; label.style.left = '8px'; label.style.top = `${originY - y * cell - 5}px`; grid.append(label);
  }
  const occupied = new Map();
  for (const [name, robot] of robots) {
    const key = `${robot.x},${robot.y}`; const peers = occupied.get(key) || []; peers.push(name); occupied.set(key, peers);
  }
  for (const [name, robot] of robots) {
    const peers = occupied.get(`${robot.x},${robot.y}`), index = peers.indexOf(name);
    const offset = peers.length > 1 ? (index - (peers.length - 1) / 2) * Math.min(24, cell / peers.length) : 0;
    const button = document.createElement('button'); button.className = `robot${selected === name ? ' selected' : ''}`;
    button.style.setProperty('--team', color(robot.team_id));
    button.style.left = `${originX + robot.x * cell - 20 + offset}px`; button.style.top = `${originY - robot.y * cell - 20 + offset}px`;
    button.innerHTML = icon; button.title = `${name} · Team ${robot.team_id} · (${robot.x}, ${robot.y})`;
    button.setAttribute('aria-label', button.title); button.setAttribute('aria-pressed', String(selected === name));
    const label = document.createElement('span'); label.className = 'robot-label'; label.textContent = name; button.append(label);
    button.onclick = () => select(name); grid.append(button);
  }
}
function meter(label, value, max, tint) {
  const wrapper = document.createElement('div'), caption = document.createElement('div'); caption.className = 'meter-label';
  const title = document.createElement('span'); title.textContent = label;
  const number = document.createElement('strong'); number.textContent = `${value} / ${max}`; caption.append(title, number);
  const bar = document.createElement('div'); bar.className = 'meter'; const fill = document.createElement('span'); fill.style.width = `${Math.max(0, Math.min(100, value / max * 100))}%`; fill.style.background = tint; bar.append(fill); wrapper.append(caption, bar); return wrapper;
}
function renderControls() {
  const picker = $('robot-select');
  picker.replaceChildren();
  if (!selected || !availableNames.size) picker.add(new Option(availableNames.size ? 'Roboter auswählen' : 'Keine Roboter vorhanden', ''));
  for (const name of [...availableNames].sort((a, b) => a.localeCompare(b))) picker.add(new Option(name, name));
  picker.value = selected || ''; picker.disabled = busy || !availableNames.size;
  const robot = robots.get(selected); $('no-selection').hidden = !!robot; $('selected-controls').hidden = !robot;
  if (!robot) return;
  $('selected-controls').style.setProperty('--team', color(robot.team_id));
  $('selected-icon').innerHTML = icon; $('selected-name').textContent = selected; $('selected-team').textContent = `Team ${robot.team_id}`;
  $('position').textContent = `X ${robot.x} / Y ${robot.y}`;
  $('meters').replaceChildren(meter('Integrität', robot.health, 30, '#73c7ff'), meter('Batterie', robot.battery, 10, '#bef46a'));
}
function renderTeams() {
  const teams = new Map(); for (const [name, robot] of robots) { if (!teams.has(robot.team_id)) teams.set(robot.team_id, []); teams.get(robot.team_id).push([name, robot]); }
  $('team-count').textContent = teams.size; $('teams').replaceChildren();
  if (!teams.size) { const empty = document.createElement('p'); empty.className = 'muted'; empty.textContent = 'Noch keine Teams. Füge einen Roboter hinzu, um loszulegen.'; $('teams').append(empty); }
  for (const [id, members] of [...teams].sort((a, b) => a[0] - b[0])) {
    const team = document.createElement('div'); team.className = 'team'; team.style.setProperty('--team', color(id));
    const title = document.createElement('div'); title.className = 'team-title'; const dot = document.createElement('i'), heading = document.createElement('strong'), count = document.createElement('span');
    heading.textContent = `Team ${id}`; count.textContent = `${members.length} BOT${members.length === 1 ? '' : 'S'}`; title.append(dot, heading, count); team.append(title);
    for (const [name, robot] of members) {
      const button = document.createElement('button'); button.className = `roster${selected === name ? ' active' : ''}`;
      const text = document.createElement('span'); text.textContent = name;
      const coordinates = document.createElement('small'); coordinates.textContent = `${robot.x}, ${robot.y}`;
      button.append(text, coordinates); button.onclick = () => { select(name); boardFit = false; center = { x: robot.x, y: robot.y }; renderArena(); }; team.append(button);
    }
    $('teams').append(team);
  }
}
function integer(id) { const input = $(id); if (!input.reportValidity()) throw new Error('Bitte eine positive ganze Zahl eingeben.'); return Number(input.value); }
async function command(path, extra = {}, method = 'PUT') {
  if (busy || !robots.has(selected)) return;
  const name = selected; busy = true; document.querySelectorAll('#selected-controls button').forEach(button => button.disabled = true);
  if (path === 'rename') pendingRename = extra.new_name;
  try {
    await request(path, method, { name, ...extra });
    // HTTP confirms receipt; only SSE confirms a game action was actually applied.
    log(`${name}: Befehl „${path}“ gesendet`);
    if (path !== 'remove' && path !== 'rename') await load(name);
  } catch (error) { pendingRename = null; toast(error.message, true); throw error; }
  finally { busy = false; document.querySelectorAll('#selected-controls button').forEach(button => button.disabled = false); }
}
function move(direction) { try { command('move', { direction, steps: integer('steps') }).catch(() => {}); } catch (error) { toast(error.message, true); } }
document.querySelectorAll('[data-direction]').forEach(button => button.onclick = () => move(button.dataset.direction));
$('arena').onkeydown = event => { const direction = { ArrowUp: 'up', ArrowDown: 'down', ArrowLeft: 'left', ArrowRight: 'right' }[event.key]; if (direction) { event.preventDefault(); if (!event.repeat) move(direction); } };
$('attack').onclick = () => { try { command('attack', { power: integer('power') }).catch(() => {}); } catch (error) { toast(error.message, true); } };
$('repair').onclick = () => command('repair').catch(() => {}); $('transfer').onclick = () => command('transfer_charge').catch(() => {});
$('zoom-in').onclick = () => { boardFit = false; cell = Math.min(100, cell + 8); renderArena(); }; $('zoom-out').onclick = () => { boardFit = false; cell = Math.max(20, cell - 8); renderArena(); };
$('center').onclick = () => { boardFit = false; const robot = robots.get(selected); center = robot ? { x: robot.x, y: robot.y } : { x: 0, y: 0 }; renderArena(); };
$('fit-board').onclick = fitBoard;
new ResizeObserver(() => board && boardFit ? fitBoard() : renderArena()).observe($('arena'));
function openDialog(mode) {
  dialogMode = mode; $('dialog-title').textContent = mode === 'add' ? 'Roboter erstellen' : 'Roboter umbenennen';
  $('dialog-description').textContent = mode === 'add' ? 'Gib deinem neuen Roboter einen Namen.' : `Neuer Name für ${selected}.`;
  $('save-robot').textContent = mode === 'add' ? 'Erstellen' : 'Umbenennen';
  $('robot-name').value = mode === 'add' ? '' : selected; $('robot-dialog').showModal(); $('robot-name').focus();
}
$('robot-select').onchange = async event => {
  const name = event.target.value; if (!name) return;
  select(name);
  if (!robots.has(name)) { try { await load(name); } catch (error) { toast(error.message, true); } }
  const robot = robots.get(name); if (robot) { boardFit = false; center = { x: robot.x, y: robot.y }; renderArena(); }
};
$('add-open').onclick = () => openDialog('add'); $('rename-open').onclick = () => openDialog('rename'); $('dialog-cancel').onclick = () => $('robot-dialog').close();
$('robot-form').onsubmit = async event => {
  event.preventDefault(); const name = $('robot-name').value.trim(); if (!name) return;
  const buttons = [...$('robot-form').querySelectorAll('button')]; buttons.forEach(button => button.disabled = true);
  try {
    if (dialogMode === 'rename') await command('rename', { new_name: name });
    else { await request('create', 'POST', { name }); log(`${name}: Erstellung angefragt`); }
    $('robot-dialog').close();
  } catch (error) { toast(error.message, true); }
  finally { buttons.forEach(button => button.disabled = false); }
};
$('remove-open').onclick = () => { $('remove-description').textContent = `${selected} wird aus dem Spiel entfernt.`; $('remove-dialog').showModal(); };
$('remove-dialog').addEventListener('close', () => { if ($('remove-dialog').returnValue === 'remove') command('remove', {}, 'DELETE').catch(() => {}); });
const stream = new EventSource('/event');
let connectedOnce = false;
stream.onopen = () => {
  $('connection').textContent = 'Live verbunden'; $('status-dot').classList.add('online');
  log(connectedOnce ? 'Live-Verbindung wiederhergestellt' : 'Live-Verbindung hergestellt');
  if (connectedOnce) toast('Verbindung wiederhergestellt. Verpasste Ereignisse können nicht nachgeladen werden.');
  connectedOnce = true;
};
stream.onerror = () => { $('connection').textContent = 'Verbindung unterbrochen · neuer Versuch …'; $('status-dot').classList.remove('online'); };
stream.addEventListener('robot_state', event => {
  try {
    const { name, state, event: kind } = JSON.parse(event.data);
    if (typeof name !== 'string' || typeof kind !== 'string' || !validState(state)) throw new Error('Ungültiges SSE-Ereignis');
    if (['removed', 'remove', 'delete', 'deleted'].includes(kind.toLowerCase())) {
      availableNames.delete(name); robots.delete(name); if (selected === name) selected = robots.keys().next().value || null; remember(); render();
    } else {
      if (kind.toLowerCase() === 'rename' && name === pendingRename) { selected = name; pendingRename = null; }
      update(name, state);
    }
    const labels = { Created: 'erstellt', Removed: 'entfernt', Rename: 'umbenannt', Changed: 'aktualisiert' };
    log(`${name} ${labels[kind] || kind} · (${state.x}, ${state.y})`, state.team_id);
  } catch (error) { toast(error.message, true); }
});
render();
async function loadBoard() {
  await Promise.allSettled([
    (async () => {
      try {
        const dimensions = await request('board_dimensions', 'GET');
        if (!dimensions || !Number.isInteger(dimensions.x) || !Number.isInteger(dimensions.y) || dimensions.x <= 0 || dimensions.y <= 0) throw new Error('Ungültige Spielfeldgröße');
        board = dimensions; $('board-size').textContent = `${board.x} × ${board.y} Felder`;
        $('fit-board').disabled = false; fitBoard();
      } catch (error) { $('board-size').textContent = 'Spielfeldgröße nicht verfügbar'; log(error.message); }
    })(),
    (async () => {
      try {
        const pads = await request('chargepads', 'GET');
        if (!Array.isArray(pads) || !pads.every(pad => pad && Number.isInteger(pad.x) && Number.isInteger(pad.y))) throw new Error('Ungültige Ladefelder');
        chargepads = pads; $('pad-count').textContent = `${pads.length} Ladefelder`; renderArena();
      } catch (error) { $('pad-count').textContent = 'Ladefelder nicht verfügbar'; log(error.message); }
    })(),
  ]);
}
loadBoard();
// Optional discovery endpoint; older API versions only support explicit names.
async function initialize() {
  let names = known;
  try { const list = await request('robot_list', 'GET'); if (Array.isArray(list)) names = list; } catch { /* Older API. */ }
  names = names.filter(name => typeof name === 'string');
  for (const name of names) availableNames.add(name);
  renderControls();
  for (const name of names) {
    try { await load(name); } catch { log(`${name} konnte nicht geladen werden. Zustand noch nicht verfügbar; Live-Ereignisse werden übernommen.`); }
  }
}
initialize();
