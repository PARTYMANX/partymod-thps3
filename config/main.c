#include <stdio.h>

#include <Windows.h>
#include <windowsx.h>
#include <CommCtrl.h>
#include <uxtheme.h>

#include <SDL2/SDL.h>

// configuration application for PARTYMOD

// gui library
// this could probably be moved to a different file but i'm too lazy lol

#pragma comment(linker,"\"/manifestdependency:type='win32' \
name='Microsoft.Windows.Common-Controls' version='6.0.0.0' \
processorArchitecture='*' publicKeyToken='6595b64144ccf1df' language='*'\"")

struct pgui_control;

typedef void (*control_callback)(struct pgui_control *, void *);
typedef void (*control_value_callback)(struct pgui_control *, int, void *);
//typedef void (*control_string_callback)(char *, void *);

typedef enum {
	PGUI_CONTROL_TYPE_WINDOW,
	PGUI_CONTROL_TYPE_EMPTY,
	PGUI_CONTROL_TYPE_BUTTON,
	PGUI_CONTROL_TYPE_CHECKBOX,
	PGUI_CONTROL_TYPE_COMBOBOX,
	PGUI_CONTROL_TYPE_GROUPBOX,
	PGUI_CONTROL_TYPE_TABS,
	PGUI_CONTROL_TYPE_TEXTBOX,
	PGUI_CONTROL_TYPE_LABEL,
} pgui_controltype;

typedef enum {
	PGUI_LABEL_JUSTIFY_LEFT,
	PGUI_LABEL_JUSTIFY_CENTER,
} pgui_label_justify;

typedef struct {
	int current_id;
} pgui_control_window;

typedef struct {
	control_callback onPressed;
	void *onPressedData;
} pgui_control_button;

typedef struct {
	control_value_callback on_toggle;
	void *on_toggle_data;
} pgui_control_checkbox;

typedef struct {
	control_value_callback on_select;
	void *on_select_data;
} pgui_control_combobox;

typedef struct {
	int num_tabs;
	int current_tab;
	HBRUSH brush;	// used for drawing; brush with the tab's visuals
} pgui_control_tabs;

typedef struct {
	control_callback on_focus_lost;
	void *on_focus_lost_data;
	control_callback on_focus_gained;
	void *on_focus_gained_data;
} pgui_control_textbox;

typedef struct pgui_control {
	pgui_controltype type;
	int id;
	HWND hwnd;
	size_t num_children;
	size_t children_size;

	int x;
	int y;
	int w;
	int h;

	int hidden;

	struct pgui_control **children;
	struct pgui_control *parent;
	union {
		pgui_control_window window;
		pgui_control_button button;
		pgui_control_checkbox checkbox;
		pgui_control_combobox combobox;
		pgui_control_tabs tabs;
		pgui_control_textbox textbox;
	};
} pgui_control;

int pgui_initialized = 0;
size_t num_windows = 0;
size_t window_list_size = 0;
pgui_control **window_list = NULL;

HINSTANCE hinst;
HFONT hfont;

LRESULT pgui_control_wndproc(pgui_control *control, HWND hwnd, UINT uMsg, WPARAM wParam, LPARAM lParam);

void pgui_add_child(pgui_control *parent, pgui_control *child) {
	if (!parent->children) {
		parent->children = malloc(sizeof(pgui_control *));

		parent->children_size = 1;
	} else if (parent->children_size <= parent->num_children) {
		parent->children = realloc(parent->children, sizeof(pgui_control *) * (parent->children_size + 1));

		parent->children_size += 1;
	}

	parent->children[parent->num_children] = child;

	parent->num_children++;
}

void pgui_control_hide(pgui_control *control, int hidden) {
	if (!hidden || !control->hidden) {
		ShowWindow(control->hwnd, !hidden);
	} else {
		ShowWindow(control->hwnd, 0);
	}

	for (int i = 0; i < control->num_children; i++) {
		pgui_control_hide(control->children[i], hidden);
	}
}

void pgui_control_set_hidden(pgui_control *control, int hidden) {
	control->hidden = hidden;

	pgui_control_hide(control, hidden);
}

void pgui_get_hierarchy_position(pgui_control *control, int *x, int *y) {
	if (control->parent) {
		*x += control->parent->x;
		*y += control->parent->y;
		pgui_get_hierarchy_position(control->parent, x, y);
	}
}

pgui_control *pgui_create_control(int x, int y, int w, int h, pgui_control *parent) {
	pgui_control *result = malloc(sizeof(pgui_control));
	result->num_children = 0;
	result->children = NULL;
	result->parent = parent;

	pgui_add_child(parent, result);

	result->x = x;
	result->y = y;
	result->w = w;
	result->h = h;

	return result;
}

pgui_control *findControl(pgui_control *control, HWND target) {
	if (control->hwnd == target) {
		return control;
	} else if (control->num_children == 0) {
		return NULL;
	}

	for (int i = 0; i < control->num_children; i++) {
		pgui_control *result = findControl(control->children[i], target);
		if (result) {
			return result;
		}
	}

	return NULL;
}

LRESULT CALLBACK pgui_wndproc(HWND hwnd, UINT uMsg, WPARAM wParam, LPARAM lParam) {

	// find hwnd in window list
	pgui_control *self = NULL;

	for (int i = 0; i < num_windows; i++) {
		//printf("hwnd in = 0x%08x, checking 0x%08x\n", hwnd, window_list[i].hwnd);
		if (window_list[i]->hwnd == hwnd) {
			//printf("window found!");
			self = window_list[i];
			break;
		}
	}
	
	switch (uMsg) {
		case WM_DESTROY: {
			PostQuitMessage(0);
			return 0;
		}
		case WM_PAINT: {
			PAINTSTRUCT ps;
			HDC hdc = BeginPaint(hwnd, &ps);

			// All painting occurs here, between BeginPaint and EndPaint.

			FillRect(hdc, &ps.rcPaint, (HBRUSH) (COLOR_WINDOW));

			EndPaint(hwnd, &ps);

			return 0;
		}
		case WM_CTLCOLOR:
		case WM_CTLCOLORBTN:
		case WM_CTLCOLORSTATIC: {
			if (self) {
				// find this control
				pgui_control *child_control = findControl(self, (HWND)lParam);
				
				if (child_control) {
					// find parent tab if it exists
					pgui_control *parent_tab = child_control->parent;
					while (parent_tab && parent_tab->type != PGUI_CONTROL_TYPE_TABS) {
						parent_tab = parent_tab->parent;
					}

					if (parent_tab) {
						// create a brush that copies the tab's body 
						if (!parent_tab->tabs.brush) {
							RECT rc;

							GetWindowRect(parent_tab->hwnd, &rc);
							HDC hdc = GetDC(parent_tab->hwnd);
							HDC hdc_new = CreateCompatibleDC(hdc);	// create a new device context to draw our tab into
							HBITMAP hbmp = CreateCompatibleBitmap(hdc, rc.right - rc.left, rc.bottom - rc.top);	// create a new bitmap to draw the tab into
							HBITMAP hbmp_old = (HBITMAP)(SelectObject(hdc_new, hbmp));	// replace the device context's bitmap with our new bitmap

							SendMessage(parent_tab->hwnd, WM_PRINTCLIENT, hdc_new, (LPARAM)(PRF_ERASEBKGND | PRF_CLIENT | PRF_NONCLIENT));	// draw the tab into our bitmap
							parent_tab->tabs.brush = CreatePatternBrush(hbmp);	// create a brush from the bitmap
							SelectObject(hdc_new, hbmp_old);	// replace the bitmap in the device context

							DeleteObject(hbmp);
							DeleteDC(hdc_new);
							ReleaseDC(parent_tab->hwnd, hdc);
						}

						// use our previously created brush as a background for the control
						RECT rc2;

						HDC hEdit = (HDC)wParam;
						SetBkMode(hEdit, TRANSPARENT);

						GetWindowRect((HWND)lParam, &rc2);	// get control's position
						MapWindowPoints(NULL, parent_tab->hwnd, (LPPOINT)(&rc2), 2);	// convert coordinates into tab's space
						SetBrushOrgEx(hEdit, -rc2.left, -rc2.top, NULL);	// set brush origin to our control's position

						return parent_tab->tabs.brush;
					}
				}
			}
		}
	}

	if (self) {
		pgui_control_wndproc(self, hwnd, uMsg, wParam, lParam);
	}

	return DefWindowProc(hwnd, uMsg, wParam, lParam);
}

pgui_control *pgui_window_create(int width, int height, char *title) {	// TODO: window styling
	pgui_control *result = malloc(sizeof(pgui_control));
	result->type = PGUI_CONTROL_TYPE_WINDOW;
	result->id = 0;
	result->num_children = 0;
	result->children = NULL;
	result->parent = NULL;
	result->window.current_id = 0x8800;
	result->x = 0;
	result->y = 0;
	result->w = width;
	result->h = height;

	HINSTANCE hinst = GetModuleHandle(NULL);

	const wchar_t CLASS_NAME[]  = L"PGUI Window Class";

	if (!pgui_initialized) {
		INITCOMMONCONTROLSEX icex;
		icex.dwSize = sizeof(INITCOMMONCONTROLSEX);
		icex.dwICC = ICC_TAB_CLASSES;
		InitCommonControlsEx(&icex);

		LOGFONT lf;
		GetObject (GetStockObject(DEFAULT_GUI_FONT), sizeof(LOGFONT), &lf); 
		hfont = CreateFont (lf.lfHeight, lf.lfWidth, 
			lf.lfEscapement, lf.lfOrientation, lf.lfWeight, 
			lf.lfItalic, lf.lfUnderline, lf.lfStrikeOut, lf.lfCharSet, 
			lf.lfOutPrecision, lf.lfClipPrecision, lf.lfQuality, 
			lf.lfPitchAndFamily, lf.lfFaceName); 

		WNDCLASS wc = { 0,
			pgui_wndproc,
			0,
			0,
			hinst,
			NULL,
			NULL,
			NULL,
			NULL,
			CLASS_NAME
		};

		RegisterClass(&wc);
	}

	result->hwnd = CreateWindow(CLASS_NAME, title, WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX, CW_USEDEFAULT, CW_USEDEFAULT, width, height, NULL, NULL, hinst, NULL);
	SendMessage(result->hwnd, WM_SETFONT, (WPARAM)hfont, TRUE);

	  RECT rc_client, rc_window;
	  POINT pt_diff;
	  GetClientRect(result->hwnd, &rc_client);
	  GetWindowRect(result->hwnd, &rc_window);
	  pt_diff.x = (rc_window.right - rc_window.left) - rc_client.right;
	  pt_diff.y = (rc_window.bottom - rc_window.top) - rc_client.bottom;
	  MoveWindow(result->hwnd, rc_window.left, rc_window.top, width + pt_diff.x, height + pt_diff.y, TRUE);

	// todo: add window to window list, return window
	if (!window_list) {
		window_list = malloc(sizeof(pgui_control *));

		window_list_size = 1;
	} else if (window_list_size == num_windows) {
		window_list = realloc(window_list, sizeof(pgui_control *) * (window_list_size + 1));

		window_list_size += 1;
	}

	window_list[num_windows] = result;
	num_windows += 1;

	return result;
}

void pgui_window_show(pgui_control *control) {
	ShowWindow(control->hwnd, SW_NORMAL);
}

pgui_control *pgui_empty_create(int x, int y, int w, int h, pgui_control *parent) {	// TODO: window styling
	pgui_control *result = pgui_create_control(x, y, w, h, parent);
	result->type = PGUI_CONTROL_TYPE_EMPTY;

	pgui_get_hierarchy_position(result, &x, &y);

	// get window hwnd
	pgui_control *node = parent;
	while (node->parent) {
		node = node->parent;
	}

	result->id = 0;

	result->hwnd = NULL;

	// reorder parent(s) probably?  need to remember how win32 orders things

	return result;
}

LRESULT pgui_button_wndproc(pgui_control *control, HWND hwnd, UINT uMsg, WPARAM wParam, LPARAM lParam) {
	switch (uMsg) {
		case WM_COMMAND: {
			//printf("COMMAND CHILD!!! LO: %d, HI: %d, L: %d\n", LOWORD(wParam), HIWORD(wParam), lParam);
			int controlId = LOWORD(wParam);

			if (controlId == control->id && control->button.onPressed) {
				control->button.onPressed(control, control->button.onPressedData);	// maybe pass in self?
			}

			return 0;
		}
	}
}

pgui_control *pgui_button_create(int x, int y, int w, int h, char *label, pgui_control *parent) {	// TODO: window styling
	pgui_control *result = pgui_create_control(x, y, w, h, parent);
	result->type = PGUI_CONTROL_TYPE_BUTTON;

	result->button.onPressed = NULL;
	result->button.onPressedData = NULL;

	pgui_get_hierarchy_position(result, &x, &y);

	// get window hwnd
	pgui_control *node = parent;
	while (node->parent) {
		node = node->parent;
	}

	result->id = node->window.current_id;

	result->hwnd = CreateWindow(WC_BUTTON, label, WS_TABSTOP | WS_VISIBLE | WS_CHILD | BS_PUSHBUTTON, x, y, w, h, node->hwnd, node->window.current_id, hinst, NULL);
	SendMessage(result->hwnd, WM_SETFONT, (WPARAM)hfont, TRUE);

	// reorder parent(s) probably?  need to remember how win32 orders things

	node->window.current_id += 1;

	return result;
}

void pgui_button_set_on_press(pgui_control *control, control_callback on_pressed, void *data) {
	control->button.onPressed = on_pressed;
	control->button.onPressedData = data;
}

void pgui_button_set_enabled(pgui_control *control, int enabled) {
	Button_Enable(control->hwnd, enabled);
}

LRESULT pgui_checkbox_wndproc(pgui_control *control, HWND hwnd, UINT uMsg, WPARAM wParam, LPARAM lParam) {
	switch (uMsg) {
		case WM_COMMAND: {
			//printf("COMMAND CHILD!!! LO: %d, HI: %d, L: %d\n", LOWORD(wParam), HIWORD(wParam), lParam);
			int controlId = LOWORD(wParam);

			if (controlId == control->id) {
				int checkstate = SendMessage(control->hwnd, BM_GETCHECK, 0, 0) == BST_CHECKED;

				if (checkstate) {
					SendMessage(control->hwnd, BM_SETCHECK, BST_UNCHECKED, 0);
				} else {
					SendMessage(control->hwnd, BM_SETCHECK, BST_CHECKED, 0);
				}

				if (control->checkbox.on_toggle) {
					control->checkbox.on_toggle(control, !checkstate, control->checkbox.on_toggle_data);	// maybe pass in self?
				}
			}

			return 0;
		}
	}
}

pgui_control *pgui_checkbox_create(int x, int y, int w, int h, char *label, pgui_control *parent) {	// TODO: window styling
	pgui_control *result = pgui_create_control(x, y, w, h, parent);
	result->type = PGUI_CONTROL_TYPE_CHECKBOX;

	result->checkbox.on_toggle = NULL;
	result->checkbox.on_toggle_data = NULL;

	pgui_get_hierarchy_position(result, &x, &y);

	// get window hwnd
	pgui_control *node = parent;
	while (node->parent) {
		node = node->parent;
	}

	result->id = node->window.current_id;

	result->hwnd = CreateWindow(WC_BUTTON, label, WS_TABSTOP | WS_VISIBLE | WS_CHILD | BS_CHECKBOX, x, y, w, h, node->hwnd, node->window.current_id, hinst, NULL);
	SendMessage(result->hwnd, WM_SETFONT, (WPARAM)hfont, TRUE);

	// reorder parent(s) probably?  need to remember how win32 orders things

	node->window.current_id += 1;

	return result;
}

void pgui_checkbox_set_on_toggle(pgui_control *control, control_value_callback on_toggle, void *data) {
	control->checkbox.on_toggle = on_toggle;
	control->checkbox.on_toggle_data = data;
}

void pgui_checkbox_set_checked(pgui_control *control, int checked) {
	if (!checked) {
		SendMessage(control->hwnd, BM_SETCHECK, BST_UNCHECKED, 0);
	} else {
		SendMessage(control->hwnd, BM_SETCHECK, BST_CHECKED, 0);
	}
}

void pgui_checkbox_set_enabled(pgui_control *control, int enabled) {
	Button_Enable(control->hwnd, enabled);
}

pgui_control *pgui_label_create(int x, int y, int w, int h, char *label, pgui_label_justify justify, pgui_control *parent) {	// todo: positioning (center, left, etc)
	pgui_control *result = pgui_create_control(x, y, w, h, parent);
	result->type = PGUI_CONTROL_TYPE_LABEL;

	pgui_get_hierarchy_position(result, &x, &y);

	// get window hwnd
	pgui_control *node = parent;
	while (node->parent) {
		node = node->parent;
	}

	int justify_flag = 0;
	if (justify == PGUI_LABEL_JUSTIFY_CENTER) {
		justify_flag = SS_CENTER;
	}

	result->hwnd = CreateWindow(WC_STATIC, label, WS_CHILD | WS_VISIBLE | justify_flag, x, y, w, h, node->hwnd, NULL, hinst, NULL);
	SendMessage(result->hwnd, WM_SETFONT, (WPARAM)hfont, TRUE);

	// reorder parent(s) probably?  need to remember how win32 orders things

	return result;
}

void pgui_label_set_enabled(pgui_control *control, int enabled) {
	Static_Enable(control->hwnd, enabled);
}

pgui_control *pgui_groupbox_create(int x, int y, int w, int h, char *label, pgui_control *parent) {	// todo: positioning (center, left, etc)
	pgui_control *result = pgui_create_control(x, y, w, h, parent);
	result->type = PGUI_CONTROL_TYPE_GROUPBOX;

	pgui_get_hierarchy_position(result, &x, &y);

	// get window hwnd
	pgui_control *node = parent;
	while (node->parent) {
		node = node->parent;
	}

	result->hwnd = CreateWindow(WC_BUTTON, label, WS_TABSTOP | WS_VISIBLE | WS_CHILD | BS_GROUPBOX, x, y, w, h, node->hwnd, NULL, hinst, NULL);
	SendMessage(result->hwnd, WM_SETFONT, (WPARAM)hfont, TRUE);

	// reorder parent(s) probably?  need to remember how win32 orders things

	return result;
}

pgui_control *pgui_textbox_create(int x, int y, int w, int h, char *text, pgui_control *parent) {	// todo: positioning (center, left, etc)
	pgui_control *result = pgui_create_control(x, y, w, h, parent);
	result->type = PGUI_CONTROL_TYPE_TEXTBOX;

	result->textbox.on_focus_gained = NULL;
	result->textbox.on_focus_gained_data = NULL;
	result->textbox.on_focus_lost = NULL;
	result->textbox.on_focus_lost_data = NULL;

	pgui_get_hierarchy_position(result, &x, &y);

	// get window hwnd
	pgui_control *node = parent;
	while (node->parent) {
		node = node->parent;
	}

	result->id = node->window.current_id;

	result->hwnd = CreateWindowEx(WS_EX_CLIENTEDGE, WC_EDIT, text, WS_CHILD | WS_VISIBLE, x, y, w, h, node->hwnd, (HMENU)node->window.current_id, hinst, NULL);
	SendMessage(result->hwnd, WM_SETFONT, (WPARAM)hfont, TRUE);

	// reorder parent(s) probably?  need to remember how win32 orders things

	node->window.current_id += 1;

	return result;
}

void pgui_textbox_set_on_focus_gained(pgui_control *control, control_value_callback callback, void *data) {
	control->textbox.on_focus_gained = callback;
	control->textbox.on_focus_gained_data = data;
}

void pgui_textbox_set_on_focus_lost(pgui_control *control, control_value_callback callback, void *data) {
	control->textbox.on_focus_lost = callback;
	control->textbox.on_focus_lost_data = data;
}

void pgui_textbox_set_text(pgui_control *control, char *text) {
	Edit_SetText(control->hwnd, text);
}

void pgui_textbox_get_text(pgui_control *control, char *output_buffer, size_t buffer_size) {
	Edit_GetText(control->hwnd, output_buffer, buffer_size);
}

void pgui_textbox_set_enabled(pgui_control *control, int enabled) {
	Edit_Enable(control->hwnd, enabled);
}

LRESULT pgui_textbox_wndproc(pgui_control *control, HWND hwnd, UINT uMsg, WPARAM wParam, LPARAM lParam) {
	switch (uMsg) {
		case WM_COMMAND: {
			int controlId = LOWORD(wParam);
			int controlCode = HIWORD(wParam);

			if (controlId == control->id) {
				if (controlCode == EN_SETFOCUS) {
					if (control->textbox.on_focus_gained) {
						control->textbox.on_focus_gained(control, control->textbox.on_focus_gained_data);
					}
				}
				if (controlCode == EN_KILLFOCUS) {
					if (control->textbox.on_focus_lost) {
						control->textbox.on_focus_lost(control, control->textbox.on_focus_lost_data);
					}
				}
			}

			return 0;
		}
	}
}

pgui_control *pgui_combobox_create(int x, int y, int w, int h, char **options, size_t num_options, pgui_control *parent) {	// todo: positioning (center, left, etc)
	pgui_control *result = pgui_create_control(x, y, w, h, parent);
	result->type = PGUI_CONTROL_TYPE_COMBOBOX;

	result->combobox.on_select = NULL;
	result->combobox.on_select_data = NULL;

	pgui_get_hierarchy_position(result, &x, &y);

	// get window hwnd
	pgui_control *node = parent;
	while (node->parent) {
		node = node->parent;
	}

	result->id = node->window.current_id;

	result->hwnd = CreateWindow(WC_COMBOBOX, "", WS_TABSTOP | WS_VISIBLE | WS_CHILD | CBS_DROPDOWNLIST, x, y, w, h, node->hwnd, node->window.current_id, hinst, NULL);
	SendMessage(result->hwnd, WM_SETFONT, (WPARAM)hfont, TRUE);

	for (int i = 0; i < num_options; i++) {
		ComboBox_AddString(result->hwnd, options[i]);
	}

	ComboBox_SetCurSel(result->hwnd, 0);

	// reorder parent(s) probably?  need to remember how win32 orders things

	node->window.current_id += 1;

	return result;
}

LRESULT pgui_combobox_wndproc(pgui_control *control, HWND hwnd, UINT uMsg, WPARAM wParam, LPARAM lParam) {
	switch (uMsg) {
		case WM_COMMAND: {
			int controlId = LOWORD(wParam);
			int controlCode = HIWORD(wParam);

			if (controlId == control->id) {
				if (controlCode == 1) {
					int idx = ComboBox_GetCurSel(lParam);
					if (control->combobox.on_select) {
						control->combobox.on_select(control, idx, control->combobox.on_select_data);
					}
				}
			}

			return 0;
		}
	}
}

void pgui_combobox_set_on_select(pgui_control *control, control_value_callback on_select, void *data) {
	control->combobox.on_select = on_select;
	control->combobox.on_select_data = data;
}

void pgui_combobox_set_selection(pgui_control *control, int value) {
	ComboBox_SetCurSel(control->hwnd, value);
}

int pgui_combobox_get_selection(pgui_control *control) {
	return ComboBox_GetCurSel(control->hwnd);
}

void pgui_combobox_set_enabled(pgui_control *control, int enabled) {
	ComboBox_Enable(control->hwnd, enabled);
}

pgui_control *pgui_tabs_create(int x, int y, int w, int h, char **options, size_t num_options, pgui_control *parent) {	// todo: positioning (center, left, etc)
	pgui_control *result = pgui_create_control(x, y, w, h, parent);
	result->type = PGUI_CONTROL_TYPE_TABS;

	result->tabs.current_tab = 0;
	result->tabs.num_tabs = num_options;
	result->tabs.brush = NULL;

	pgui_get_hierarchy_position(result, &x, &y);

	// get window hwnd
	pgui_control *node = parent;
	while (node->parent) {
		node = node->parent;
	}

	result->id = node->window.current_id;

	result->hwnd = CreateWindow(WC_TABCONTROL, "", WS_CHILD | WS_VISIBLE, x, y, w, h, node->hwnd, node->window.current_id, hinst, NULL);
	SendMessage(result->hwnd, WM_SETFONT, (WPARAM)hfont, TRUE);

	for (int i = 0; i < num_options; i++) {
		TCITEM item;
		item.mask = TCIF_TEXT | TCIF_IMAGE;
		item.iImage = -1;
	
		item.pszText = options[i];
		TabCtrl_InsertItem(result->hwnd, i, &item);
	}

	ComboBox_SetCurSel(result->hwnd, 0);

	RECT tabRect;
	GetClientRect(result->hwnd, &tabRect);
	TabCtrl_AdjustRect(result->hwnd, FALSE, &tabRect);

	// reorder parent(s) probably?  need to remember how win32 orders things

	for (int i = 0; i < num_options; i++) {
		pgui_empty_create(tabRect.left, tabRect.top, tabRect.right - tabRect.left, tabRect.bottom - tabRect.top, result);
	}

	for (int i = 1; i < num_options; i++) {
		pgui_control_set_hidden(result->children[i], 1);
	}

	node->window.current_id += 1;

	return result;
}

LRESULT pgui_tabs_wndproc(pgui_control *control, HWND hwnd, UINT uMsg, WPARAM wParam, LPARAM lParam) {
	switch (uMsg) {
		case WM_NOTIFY: {
			int controlCode = ((LPNMHDR)lParam)->code;
			int controlId = ((LPNMHDR)lParam)->idFrom;

			if (controlId == control->id) {
				if (controlCode == TCN_SELCHANGE) {
					int tab = TabCtrl_GetCurSel(((LPNMHDR)lParam)->hwndFrom);

					// hide all children on current tab
					pgui_control_set_hidden(control->children[control->tabs.current_tab], 1);

					// show all children on new tab
					pgui_control_set_hidden(control->children[tab], 0); 
					
					// set current tab to new tab
					control->tabs.current_tab = tab;
				}
			}

			return 0;
		}
	}
}

LRESULT pgui_control_wndproc(pgui_control *control, HWND hwnd, UINT uMsg, WPARAM wParam, LPARAM lParam) {
	switch(control->type) {
		case PGUI_CONTROL_TYPE_BUTTON:
			pgui_button_wndproc(control, hwnd, uMsg, wParam, lParam);
			break;
		case PGUI_CONTROL_TYPE_CHECKBOX:
			pgui_checkbox_wndproc(control, hwnd, uMsg, wParam, lParam);
			break;
		case PGUI_CONTROL_TYPE_TEXTBOX:
			pgui_textbox_wndproc(control, hwnd, uMsg, wParam, lParam);
			break;
		case PGUI_CONTROL_TYPE_COMBOBOX:
			pgui_combobox_wndproc(control, hwnd, uMsg, wParam, lParam);
			break;
		case PGUI_CONTROL_TYPE_TABS:
			pgui_tabs_wndproc(control, hwnd, uMsg, wParam, lParam);
			break;
		default:
			break;
	}

	for (int i = 0; i < control->num_children; i++) {
		pgui_control_wndproc(control->children[i], hwnd, uMsg, wParam, lParam);
	}

	return 0;
}

// config program start

#define CONFIG_FILE_NAME "partymod.ini"

struct displayMode {
	int width;
	int height;
};

int numDisplayModes;
struct displayMode *displayModeList;
char **displayModeStringList;

void initResolutionList() {
	DEVMODE deviceMode;

	numDisplayModes = 0;
	displayModeList = NULL;
	
	int i = 0;

	while (EnumDisplaySettings(NULL, i, &deviceMode)) {
		struct displayMode displayMode = { deviceMode.dmPelsWidth, deviceMode.dmPelsHeight };

		// insert into list
		if (displayModeList == NULL) {
			displayModeList = malloc(sizeof(struct displayMode));
			displayModeList[0] = displayMode;
			numDisplayModes = 1;
		} else {
			// search list for duplicate or larger resolution
			int j;
			int isDuplicate = 0;
			for (j = 0; j < numDisplayModes; j++) {
				if (displayModeList[j].width == displayMode.width && displayModeList[j].height == displayMode.height) {
					isDuplicate = 1;
					break;
				} else if (displayModeList[j].width >= displayMode.width && displayModeList[j].height >= displayMode.height) {
					break;
				}
			}

			if (!isDuplicate) {
				displayModeList = realloc(displayModeList, sizeof(struct displayMode) * (numDisplayModes + 1));
				
				// move existing elements
				for (int k = numDisplayModes; k > j; k--) {
					displayModeList[k] = displayModeList[k - 1];
				}

				displayModeList[j] = displayMode;
				numDisplayModes++;
			}
		}

		i++;
	}

	displayModeStringList = malloc(sizeof(char *) * (numDisplayModes + 1));
	displayModeStringList[0] = "Default Desktop Resolution";

	for (i = 0; i < numDisplayModes; i++) {
		//printf("OUTPUT DISPLAY MODE %d: %d x %d\n", i, displayModeList[i].width, displayModeList[i].height);
		char resolutionString[64];
		sprintf(resolutionString, "%dx%d", displayModeList[i].width, displayModeList[i].height);

		displayModeStringList[i + 1] = calloc(strlen(resolutionString) + 1, 1);
		strcpy(displayModeStringList[i + 1], resolutionString);
	}
}

struct settings {
	int resX;
	int resY;
	int windowed;
	int borderless;

	int shadows;
	int particles;
	int animating_textures;
	int distance_fog;
	int low_detail_models;

	int play_intro;
	int il_mode;
	int disable_trick_limit;
};

struct keybinds {
	SDL_Scancode ollie;
	SDL_Scancode grab;
	SDL_Scancode flip;
	SDL_Scancode grind;
	SDL_Scancode spinLeft;
	SDL_Scancode spinRight;
	SDL_Scancode nollie;
	SDL_Scancode switchRevert;
	SDL_Scancode pause;

	SDL_Scancode forward;
	SDL_Scancode backward;
	SDL_Scancode left;
	SDL_Scancode right;

	SDL_Scancode camUp;
	SDL_Scancode camDown;
	SDL_Scancode camLeft;
	SDL_Scancode camRight;
	SDL_Scancode viewToggle;
	SDL_Scancode swivelLock;
};

// a recreation of the SDL_GameControllerButton enum, but with the addition of right/left trigger
typedef enum {
	CONTROLLER_UNBOUND = -1,
	CONTROLLER_BUTTON_A = SDL_CONTROLLER_BUTTON_A,
	CONTROLLER_BUTTON_B = SDL_CONTROLLER_BUTTON_B,
	CONTROLLER_BUTTON_X = SDL_CONTROLLER_BUTTON_X,
	CONTROLLER_BUTTON_Y = SDL_CONTROLLER_BUTTON_Y,
	CONTROLLER_BUTTON_BACK = SDL_CONTROLLER_BUTTON_BACK,
	CONTROLLER_BUTTON_GUIDE = SDL_CONTROLLER_BUTTON_GUIDE,
	CONTROLLER_BUTTON_START = SDL_CONTROLLER_BUTTON_START,
	CONTROLLER_BUTTON_LEFTSTICK = SDL_CONTROLLER_BUTTON_LEFTSTICK,
	CONTROLLER_BUTTON_RIGHTSTICK = SDL_CONTROLLER_BUTTON_RIGHTSTICK,
	CONTROLLER_BUTTON_LEFTSHOULDER = SDL_CONTROLLER_BUTTON_LEFTSHOULDER,
	CONTROLLER_BUTTON_RIGHTSHOULDER = SDL_CONTROLLER_BUTTON_RIGHTSHOULDER,
	CONTROLLER_BUTTON_DPAD_UP = SDL_CONTROLLER_BUTTON_DPAD_UP,
	CONTROLLER_BUTTON_DPAD_DOWN = SDL_CONTROLLER_BUTTON_DPAD_DOWN,
	CONTROLLER_BUTTON_DPAD_LEFT = SDL_CONTROLLER_BUTTON_DPAD_LEFT,
	CONTROLLER_BUTTON_DPAD_RIGHT = SDL_CONTROLLER_BUTTON_DPAD_RIGHT,
	CONTROLLER_BUTTON_MISC1 = SDL_CONTROLLER_BUTTON_MISC1,
	CONTROLLER_BUTTON_PADDLE1 = SDL_CONTROLLER_BUTTON_PADDLE1,
	CONTROLLER_BUTTON_PADDLE2 = SDL_CONTROLLER_BUTTON_PADDLE2,
	CONTROLLER_BUTTON_PADDLE3 = SDL_CONTROLLER_BUTTON_PADDLE3,
	CONTROLLER_BUTTON_PADDLE4 = SDL_CONTROLLER_BUTTON_PADDLE4,
	CONTROLLER_BUTTON_TOUCHPAD = SDL_CONTROLLER_BUTTON_TOUCHPAD,
	CONTROLLER_BUTTON_RIGHTTRIGGER = 21,
	CONTROLLER_BUTTON_LEFTTRIGGER = 22,
} controllerButton;

typedef enum {
	CONTROLLER_STICK_UNBOUND = -1,
	CONTROLLER_STICK_LEFT = 0,
	CONTROLLER_STICK_RIGHT = 1
} controllerStick;

struct controllerbinds {
	controllerButton menu;
	controllerButton cameraToggle;
	controllerButton cameraSwivelLock;

	controllerButton grind;
	controllerButton grab;
	controllerButton ollie;
	controllerButton kick;

	controllerButton leftSpin;
	controllerButton rightSpin;
	controllerButton nollie;
	controllerButton switchRevert;

	controllerButton right;
	controllerButton left;
	controllerButton up;
	controllerButton down;

	controllerStick movement;
	controllerStick camera;
};

struct settings settings;
struct keybinds keybinds;
struct controllerbinds padbinds;

controllerButton buttonLUT[] = {
	CONTROLLER_UNBOUND,
	CONTROLLER_BUTTON_A,
	CONTROLLER_BUTTON_B,
	CONTROLLER_BUTTON_X,
	CONTROLLER_BUTTON_Y,
	CONTROLLER_BUTTON_DPAD_UP,
	CONTROLLER_BUTTON_DPAD_DOWN,
	CONTROLLER_BUTTON_DPAD_LEFT,
	CONTROLLER_BUTTON_DPAD_RIGHT,
	CONTROLLER_BUTTON_LEFTSHOULDER,
	CONTROLLER_BUTTON_RIGHTSHOULDER,
	CONTROLLER_BUTTON_LEFTTRIGGER,
	CONTROLLER_BUTTON_RIGHTTRIGGER,
	CONTROLLER_BUTTON_LEFTSTICK,
	CONTROLLER_BUTTON_RIGHTSTICK,
	CONTROLLER_BUTTON_BACK,
	CONTROLLER_BUTTON_START,
	CONTROLLER_BUTTON_TOUCHPAD,
	CONTROLLER_BUTTON_PADDLE1,
	CONTROLLER_BUTTON_PADDLE2,
	CONTROLLER_BUTTON_PADDLE3,
	CONTROLLER_BUTTON_PADDLE4,
};

controllerButton stickLUT[] = {
	CONTROLLER_STICK_UNBOUND,
	CONTROLLER_STICK_LEFT,
	CONTROLLER_STICK_RIGHT,
};

int getIniBool(char *section, char *key, int def, char *file) {
	int result = GetPrivateProfileInt(section, key, def, file);
	if (result) {
		return 1;
	} else {
		return 0;
	}
}

void writeIniBool(char *section, char *key, int val, char *file) {
	if (val) {
		WritePrivateProfileString(section, key, "1", file);
	} else {
		WritePrivateProfileString(section, key, "0", file);
	}
}

void writeIniInt(char *section, char *key, int val, char *file) {
	char buf[16];
	itoa(val, buf, 10);
	WritePrivateProfileString(section, key, buf, file);
}

void defaultSettings() {
	settings.resX = 0;
	settings.resY = 0;
	settings.windowed = 0;
	settings.borderless = 0;

	settings.shadows = 1;
	settings.particles = 1;
	settings.animating_textures = 1;
	settings.distance_fog = 0;
	settings.low_detail_models = 0;
	
	settings.play_intro = 1;
	settings.il_mode = 0;
	settings.disable_trick_limit = 0;

	keybinds.ollie = SDL_SCANCODE_KP_2;
	keybinds.grab = SDL_SCANCODE_KP_6;
	keybinds.flip = SDL_SCANCODE_KP_4;
	keybinds.grind = SDL_SCANCODE_KP_8;
	keybinds.spinLeft = SDL_SCANCODE_KP_1;
	keybinds.spinRight = SDL_SCANCODE_KP_3;
	keybinds.nollie = SDL_SCANCODE_KP_7;
	keybinds.switchRevert = SDL_SCANCODE_KP_9;
	keybinds.pause = 0;	// unbound

	keybinds.forward = SDL_SCANCODE_W;
	keybinds.backward = SDL_SCANCODE_S;
	keybinds.left = SDL_SCANCODE_A;
	keybinds.right = SDL_SCANCODE_D;

	keybinds.camUp = SDL_SCANCODE_I;
	keybinds.camDown = SDL_SCANCODE_K;
	keybinds.camLeft = SDL_SCANCODE_J;
	keybinds.camRight = SDL_SCANCODE_L;
	keybinds.viewToggle = SDL_SCANCODE_GRAVE;
	keybinds.swivelLock = 0;	// unbound

	padbinds.menu = CONTROLLER_BUTTON_START;
	padbinds.cameraToggle = CONTROLLER_BUTTON_BACK;
	padbinds.cameraSwivelLock = CONTROLLER_BUTTON_RIGHTSTICK;

	padbinds.grind = CONTROLLER_BUTTON_Y;
	padbinds.grab = CONTROLLER_BUTTON_B;
	padbinds.ollie = CONTROLLER_BUTTON_A;
	padbinds.kick = CONTROLLER_BUTTON_X;

	padbinds.leftSpin = CONTROLLER_BUTTON_LEFTSHOULDER;
	padbinds.rightSpin = CONTROLLER_BUTTON_RIGHTSHOULDER;
	padbinds.nollie = CONTROLLER_BUTTON_LEFTTRIGGER;
	padbinds.switchRevert = CONTROLLER_BUTTON_RIGHTTRIGGER;

	padbinds.right = CONTROLLER_BUTTON_DPAD_RIGHT;
	padbinds.left = CONTROLLER_BUTTON_DPAD_LEFT;
	padbinds.up = CONTROLLER_BUTTON_DPAD_UP;
	padbinds.down = CONTROLLER_BUTTON_DPAD_DOWN;

	padbinds.movement = CONTROLLER_STICK_LEFT;
	padbinds.camera = CONTROLLER_STICK_RIGHT;
}

char configFile[1024];

char *getConfigFile() {
	char executableDirectory[1024];
	int filePathBufLen = 1024;
	GetModuleFileName(NULL, &executableDirectory, filePathBufLen);

	// find last slash
	char *exe = strrchr(executableDirectory, '\\');
	if (exe) {
		*(exe + 1) = '\0';
	}

	sprintf(configFile, "%s%s", executableDirectory, CONFIG_FILE_NAME);

	return configFile;
}

void loadSettings() {
	char *configFile = getConfigFile();

	settings.resX = GetPrivateProfileInt("Graphics", "ResolutionX", 0, configFile);
	settings.resY = GetPrivateProfileInt("Graphics", "ResolutionY", 0, configFile);
	settings.windowed = getIniBool("Graphics", "Windowed", 0, configFile);
	settings.borderless = getIniBool("Graphics", "Borderless", 0, configFile);

	settings.shadows = getIniBool("Graphics", "Shadows", 1, configFile);
	settings.particles = getIniBool("Graphics", "Particles", 1, configFile);
	settings.animating_textures = getIniBool("Graphics", "AnimatedTextures", 1, configFile);
	settings.distance_fog = getIniBool("Graphics", "DistanceFog", 0, configFile);
	settings.low_detail_models = getIniBool("Graphics", "LowDetailModels", 0, configFile);

	settings.play_intro = getIniBool("Miscellaneous", "PlayIntro", 1, configFile);
	settings.il_mode = getIniBool("Miscellaneous", "ILMode", 0, configFile);
	settings.disable_trick_limit = getIniBool("Miscellaneous", "NoTrickLimit", 0, configFile);

	keybinds.ollie = GetPrivateProfileInt("Keybinds", "Ollie", SDL_SCANCODE_KP_2, configFile);
	keybinds.grab = GetPrivateProfileInt("Keybinds", "Grab", SDL_SCANCODE_KP_6, configFile);
	keybinds.flip = GetPrivateProfileInt("Keybinds", "Flip", SDL_SCANCODE_KP_4, configFile);
	keybinds.grind = GetPrivateProfileInt("Keybinds", "Grind", SDL_SCANCODE_KP_8, configFile);
	keybinds.spinLeft = GetPrivateProfileInt("Keybinds", "SpinLeft", SDL_SCANCODE_KP_1, configFile);
	keybinds.spinRight = GetPrivateProfileInt("Keybinds", "SpinRight", SDL_SCANCODE_KP_3, configFile);
	keybinds.nollie = GetPrivateProfileInt("Keybinds", "Nollie", SDL_SCANCODE_KP_7, configFile);
	keybinds.switchRevert = GetPrivateProfileInt("Keybinds", "Switch", SDL_SCANCODE_KP_9, configFile);
	keybinds.pause = GetPrivateProfileInt("Keybinds", "Pause", 0, configFile);

	keybinds.forward = GetPrivateProfileInt("Keybinds", "Forward", SDL_SCANCODE_W, configFile);
	keybinds.backward = GetPrivateProfileInt("Keybinds", "Backward", SDL_SCANCODE_S, configFile);
	keybinds.left = GetPrivateProfileInt("Keybinds", "Left", SDL_SCANCODE_A, configFile);
	keybinds.right = GetPrivateProfileInt("Keybinds", "Right", SDL_SCANCODE_D, configFile);

	keybinds.camUp = GetPrivateProfileInt("Keybinds", "CameraUp", SDL_SCANCODE_I, configFile);
	keybinds.camDown = GetPrivateProfileInt("Keybinds", "CameraDown", SDL_SCANCODE_K, configFile);
	keybinds.camLeft = GetPrivateProfileInt("Keybinds", "CameraLeft", SDL_SCANCODE_J, configFile);
	keybinds.camRight = GetPrivateProfileInt("Keybinds", "CameraRight", SDL_SCANCODE_L, configFile);
	keybinds.viewToggle = GetPrivateProfileInt("Keybinds", "ViewToggle", SDL_SCANCODE_GRAVE, configFile);
	keybinds.swivelLock = GetPrivateProfileInt("Keybinds", "SwivelLock", 0, configFile);

	padbinds.menu = GetPrivateProfileInt("Gamepad", "Pause", CONTROLLER_BUTTON_START, configFile);
	padbinds.cameraToggle = GetPrivateProfileInt("Gamepad", "ViewToggle", CONTROLLER_BUTTON_BACK, configFile);
	padbinds.cameraSwivelLock = GetPrivateProfileInt("Gamepad", "SwivelLock", CONTROLLER_BUTTON_RIGHTSTICK, configFile);

	padbinds.grind = GetPrivateProfileInt("Gamepad", "Grind", CONTROLLER_BUTTON_Y, configFile);
	padbinds.grab = GetPrivateProfileInt("Gamepad", "Grab", CONTROLLER_BUTTON_B, configFile);
	padbinds.ollie = GetPrivateProfileInt("Gamepad", "Ollie", CONTROLLER_BUTTON_A, configFile);
	padbinds.kick = GetPrivateProfileInt("Gamepad", "Flip", CONTROLLER_BUTTON_X, configFile);

	padbinds.leftSpin = GetPrivateProfileInt("Gamepad", "SpinLeft", CONTROLLER_BUTTON_LEFTSHOULDER, configFile);
	padbinds.rightSpin = GetPrivateProfileInt("Gamepad", "SpinRight", CONTROLLER_BUTTON_RIGHTSHOULDER, configFile);
	padbinds.nollie = GetPrivateProfileInt("Gamepad", "Nollie", CONTROLLER_BUTTON_LEFTTRIGGER, configFile);
	padbinds.switchRevert = GetPrivateProfileInt("Gamepad", "Switch", CONTROLLER_BUTTON_RIGHTTRIGGER, configFile);

	padbinds.right = GetPrivateProfileInt("Gamepad", "Right", CONTROLLER_BUTTON_DPAD_RIGHT, configFile);
	padbinds.left = GetPrivateProfileInt("Gamepad", "Left", CONTROLLER_BUTTON_DPAD_LEFT, configFile);
	padbinds.up = GetPrivateProfileInt("Gamepad", "Forward", CONTROLLER_BUTTON_DPAD_UP, configFile);
	padbinds.down = GetPrivateProfileInt("Gamepad", "Backward", CONTROLLER_BUTTON_DPAD_DOWN, configFile);

	padbinds.movement = GetPrivateProfileInt("Gamepad", "MovementStick", CONTROLLER_STICK_LEFT, configFile);
	padbinds.camera = GetPrivateProfileInt("Gamepad", "CameraStick", CONTROLLER_STICK_RIGHT, configFile);
}

void saveSettings() {
	char *configFile = getConfigFile();

	if (settings.resX < 640 && settings.resX != 0 && settings.resY != 0) {
		settings.resX = 640;
	}
	if (settings.resY < 480 && settings.resX != 0 && settings.resY != 0) {
		settings.resY = 480;
	}

	writeIniInt("Graphics", "ResolutionX", settings.resX, configFile);
	writeIniInt("Graphics", "ResolutionY", settings.resY, configFile);
	writeIniBool("Graphics", "Windowed", settings.windowed, configFile);
	writeIniBool("Graphics", "Borderless", settings.borderless, configFile);

	writeIniBool("Graphics", "Shadows", settings.shadows, configFile);
	writeIniBool("Graphics", "Particles", settings.particles, configFile);
	writeIniBool("Graphics", "AnimatedTextures", settings.animating_textures, configFile);
	writeIniBool("Graphics", "DistanceFog", settings.distance_fog, configFile);
	writeIniBool("Graphics", "LowDetailModels", settings.low_detail_models, configFile);

	writeIniBool("Miscellaneous", "PlayIntro", settings.play_intro, configFile);
	writeIniBool("Miscellaneous", "ILMode", settings.il_mode, configFile);
	writeIniBool("Miscellaneous", "NoTrickLimit", settings.disable_trick_limit, configFile);

	writeIniInt("Keybinds", "Ollie", keybinds.ollie, configFile);
	writeIniInt("Keybinds", "Grab", keybinds.grab, configFile);
	writeIniInt("Keybinds", "Flip", keybinds.flip, configFile);
	writeIniInt("Keybinds", "Grind", keybinds.grind, configFile);
	writeIniInt("Keybinds", "SpinLeft", keybinds.spinLeft, configFile);
	writeIniInt("Keybinds", "SpinRight", keybinds.spinRight, configFile);
	writeIniInt("Keybinds", "Nollie", keybinds.nollie, configFile);
	writeIniInt("Keybinds", "Switch", keybinds.switchRevert, configFile);
	writeIniInt("Keybinds", "Pause", keybinds.pause, configFile);

	writeIniInt("Keybinds", "Forward", keybinds.forward, configFile);
	writeIniInt("Keybinds", "Backward", keybinds.backward, configFile);
	writeIniInt("Keybinds", "Left", keybinds.left, configFile);
	writeIniInt("Keybinds", "Right", keybinds.right, configFile);

	writeIniInt("Keybinds", "CameraUp", keybinds.camUp, configFile);
	writeIniInt("Keybinds", "CameraDown", keybinds.camDown, configFile);
	writeIniInt("Keybinds", "CameraLeft", keybinds.camLeft, configFile);
	writeIniInt("Keybinds", "CameraRight", keybinds.camRight, configFile);
	writeIniInt("Keybinds", "ViewToggle", keybinds.viewToggle, configFile);
	writeIniInt("Keybinds", "SwivelLock", keybinds.swivelLock, configFile);

	writeIniInt("Gamepad", "Pause", padbinds.menu, configFile);
	writeIniInt("Gamepad", "ViewToggle", padbinds.cameraToggle, configFile);
	writeIniInt("Gamepad", "SwivelLock", padbinds.cameraSwivelLock, configFile);

	writeIniInt("Gamepad", "Grind", padbinds.grind, configFile);
	writeIniInt("Gamepad", "Grab", padbinds.grab, configFile);
	writeIniInt("Gamepad", "Ollie", padbinds.ollie, configFile);
	writeIniInt("Gamepad", "Flip", padbinds.kick, configFile);

	writeIniInt("Gamepad", "SpinLeft", padbinds.leftSpin, configFile);
	writeIniInt("Gamepad", "SpinRight", padbinds.rightSpin, configFile);
	writeIniInt("Gamepad", "Nollie", padbinds.nollie, configFile);
	writeIniInt("Gamepad", "Switch", padbinds.switchRevert, configFile);

	writeIniInt("Gamepad", "Right", padbinds.right, configFile);
	writeIniInt("Gamepad", "Left", padbinds.left, configFile);
	writeIniInt("Gamepad", "Forward", padbinds.up, configFile);
	writeIniInt("Gamepad", "Backward", padbinds.down, configFile);

	writeIniInt("Gamepad", "MovementStick", padbinds.movement, configFile);
	writeIniInt("Gamepad", "CameraStick", padbinds.camera, configFile);
}

// SDL stuff - for keybinds

void cursedSDLSetup() {
	// in order to key names, SDL has to have created a window
	// this function achieves that to initialize key binds

	SDL_Init(SDL_INIT_EVENTS);

	SDL_Window *win = SDL_CreateWindow("uhh... you're not supposed to see this", 0, 0, 0, 0, SDL_WINDOW_HIDDEN);

	SDL_DestroyWindow(win);

	SDL_Quit();
}

void setBindText(pgui_control *control, SDL_Scancode key) {
	// NOTE: requires SDL to be initialized
	if (key == 0) {
		pgui_textbox_set_text(control, "Unbound");
	} else {
		pgui_textbox_set_text(control, SDL_GetKeyName(SDL_GetKeyFromScancode(key)));
	}
}

int doing_keybind = 0;

void doKeyBind(char *name, SDL_Scancode *target, pgui_control *control) {
	if (doing_keybind) {
		// trying to do keybind while we're already trying to process one hangs the program, so let's put this off until later
		// NOTE: this causes a mildly annoying bug where selecting another control to bind fails
		return;
	}

	doing_keybind = 1;

	pgui_textbox_set_text(control, "Press key...");

	SDL_Init(SDL_INIT_EVENTS);

	char namebuf[64];
	sprintf(namebuf, "Press Key To Bind To %s", name);

	RECT windowRect;

	if (!GetWindowRect(control->hwnd, &windowRect)) {
		printf("Failed to get window rect!!\n");
		return;
	}

	// create 1x1 window in top left where it's least likely to be noticed
	SDL_Window *inputWindow = SDL_CreateWindow(namebuf, 0, 0, 1, 1, SDL_WINDOW_INPUT_FOCUS | SDL_WINDOW_BORDERLESS);

	if (!inputWindow) {
		printf("%s\n", SDL_GetError());
		SDL_Quit();
		return;
	}

	int doneInput = 0;
	SDL_Event e;
	while (!doneInput) {
		//printf("still running event loop\n");
		while(SDL_PollEvent(&e)) {
			if (e.type == SDL_KEYDOWN) {
				SDL_KeyCode keyCode = SDL_GetKeyFromScancode(e.key.keysym.scancode);
				printf("Pressed key %s\n", SDL_GetKeyName(keyCode));
				*target = e.key.keysym.scancode;
					
				doneInput = 1;
			} else if (e.type == SDL_QUIT) {
				doneInput = 1;
			} else if (e.type == SDL_WINDOWEVENT) {
				if (e.window.event == SDL_WINDOWEVENT_HIDDEN || e.window.event == SDL_WINDOWEVENT_FOCUS_LOST || e.window.event == SDL_WINDOWEVENT_MINIMIZED) {
					doneInput = 1;
				}
			}
		}
	}
	SDL_DestroyWindow(inputWindow);

	setBindText(control, *target);

	SDL_Quit();

	doing_keybind = 0;
}

char *gamepad_bind_values[22] = {
	"Unbound",
	"A/Cross",
	"B/Circle",
	"X/Square",
	"Y/Triangle",
	"D-Pad Up",
	"D-Pad Down",
	"D-Pad Left",
	"D-Pad Right",
	"LB/L1",
	"RB/R1",
	"LT/L2",
	"RT/R2",
	"Left Stick/L3",
	"Right Stick/R3",
	"Select/Back",
	"Start/Options",
	"Touchpad",
	"Paddle 1",
	"Paddle 2",
	"Paddle 3",
	"Paddle 4"
};

char *gamepad_stick_values[3] = {
	"Unbound",
	"Left Stick",
	"Right Stick"
};

struct gamepad_page {
	pgui_control *ollie;
	pgui_control *grab;
	pgui_control *flip;
	pgui_control *grind;
	pgui_control *spin_left;
	pgui_control *spin_right;
	pgui_control *nollie;
	pgui_control *switch_revert;
	pgui_control *pause;

	pgui_control *forward;
	pgui_control *backward;
	pgui_control *left;
	pgui_control *right;
	pgui_control *movement_stick;

	pgui_control *camera_stick;
	pgui_control *view_toggle;
	pgui_control *swivel_lock;
};

struct gamepad_page gamepad_page;

setButtonBindBox(pgui_control *control, controllerButton bind) {
	int sel = 0;

	switch(bind) {
	case CONTROLLER_UNBOUND:
		sel = 0;
		break;
	case CONTROLLER_BUTTON_A:
		sel = 1;
		break;
	case CONTROLLER_BUTTON_B:
		sel = 2;
		break;
	case CONTROLLER_BUTTON_X:
		sel = 3;
		break;
	case CONTROLLER_BUTTON_Y:
		sel = 4;
		break;
	case CONTROLLER_BUTTON_DPAD_UP:
		sel = 5;
		break;
	case CONTROLLER_BUTTON_DPAD_DOWN:
		sel = 6;
		break;
	case CONTROLLER_BUTTON_DPAD_LEFT:
		sel = 7;
		break;
	case CONTROLLER_BUTTON_DPAD_RIGHT:
		sel = 8;
		break;
	case CONTROLLER_BUTTON_LEFTSHOULDER:
		sel = 9;
		break;
	case CONTROLLER_BUTTON_RIGHTSHOULDER:
		sel = 10;
		break;
	case CONTROLLER_BUTTON_LEFTTRIGGER:
		sel = 11;
		break;
	case CONTROLLER_BUTTON_RIGHTTRIGGER:
		sel = 12;
		break;
	case CONTROLLER_BUTTON_LEFTSTICK:
		sel = 13;
		break;
	case CONTROLLER_BUTTON_RIGHTSTICK:
		sel = 14;
		break;
	case CONTROLLER_BUTTON_BACK:
		sel = 15;
		break;
	case CONTROLLER_BUTTON_START:
		sel = 16;
		break;
	case CONTROLLER_BUTTON_TOUCHPAD:
		sel = 17;
		break;
	case CONTROLLER_BUTTON_PADDLE1:
		sel = 18;
		break;
	case CONTROLLER_BUTTON_PADDLE2:
		sel = 19;
		break;
	case CONTROLLER_BUTTON_PADDLE3:
		sel = 20;
		break;
	case CONTROLLER_BUTTON_PADDLE4:
		sel = 21;
		break;
	}

	pgui_combobox_set_selection(control, sel);
}

setStickBindBox(pgui_control *control, controllerButton bind) {
	int sel = 0;

	switch(bind) {
	case CONTROLLER_STICK_UNBOUND:
		sel = 0;
		break;
	case CONTROLLER_STICK_LEFT:
		sel = 1;
		break;
	case CONTROLLER_STICK_RIGHT:
		sel = 2;
		break;
	}

	pgui_combobox_set_selection(control, sel);
}

void do_button_select(pgui_control *control, int value, controllerButton *target) {
	*target = buttonLUT[value];
	printf("Set button to %d\n", buttonLUT[value]);
}

void do_stick_select(pgui_control *control, int value, controllerStick *target) {
	*target = stickLUT[value];
}

pgui_control *build_button_combobox(int x, int y, int w, int h, pgui_control *parent, controllerButton *target) {
	pgui_control *result = pgui_combobox_create(x, y, w, h, gamepad_bind_values, 22, parent);
	pgui_combobox_set_on_select(result, do_button_select, target);

	return result;
}

pgui_control *build_stick_combobox(int x, int y, int w, int h, pgui_control *parent, controllerStick *target) {
	pgui_control *result = pgui_combobox_create(x, y, w, h, gamepad_stick_values, 3, parent);
	pgui_combobox_set_on_select(result, do_stick_select, target);

	return result;
}

void build_gamepad_page(pgui_control *parent) {
	pgui_control *actions_groupbox = pgui_groupbox_create(8, 8, (parent->w / 2) - 8 - 4, parent->h - 8 - 8, "Actions", parent);
	pgui_control *skater_groupbox = pgui_groupbox_create((parent->w / 2) + 4, 8, (parent->w / 2) - 8 - 4, (parent->h / 2) - 8 - 4 + 32, "Skater Controls", parent);
	pgui_control *camera_groupbox = pgui_groupbox_create((parent->w / 2) + 4, (parent->h / 2) + 4 + 32, (parent->w / 2) - 8 - 4, (parent->h / 2) - 8 - 4 - 32, "Camera Controls", parent);

	int label_offset = 4;
	int graphics_v_spacing = 39;
	int skater_v_spacing = 39;
	int camera_v_spacing = 39;
	int box_width = 80;

	// actions

	pgui_label_create(8, 16 + label_offset, 96, 16, "Ollie:", PGUI_LABEL_JUSTIFY_LEFT, actions_groupbox);
	gamepad_page.ollie = build_button_combobox(actions_groupbox->w - 8 - box_width, 16, box_width, 20, actions_groupbox, &(padbinds.ollie));

	pgui_label_create(8, 16 + label_offset + (graphics_v_spacing), 96, 16, "Grab:", PGUI_LABEL_JUSTIFY_LEFT, actions_groupbox);
	gamepad_page.grab = build_button_combobox(actions_groupbox->w - 8 - box_width, 16 + (graphics_v_spacing), box_width, 20, actions_groupbox, &(padbinds.grab));

	pgui_label_create(8, 16 + label_offset + (graphics_v_spacing * 2), 96, 16, "Flip:", PGUI_LABEL_JUSTIFY_LEFT, actions_groupbox);
	gamepad_page.flip = build_button_combobox(actions_groupbox->w - 8 - box_width, 16 + (graphics_v_spacing * 2), box_width, 20, actions_groupbox, &(padbinds.kick));

	pgui_label_create(8, 16 + label_offset + (graphics_v_spacing * 3), 96, 16, "Grind:", PGUI_LABEL_JUSTIFY_LEFT, actions_groupbox);
	gamepad_page.grind = build_button_combobox(actions_groupbox->w - 8 - box_width, 16 + (graphics_v_spacing * 3), box_width, 20, actions_groupbox, &(padbinds.grind));

	pgui_label_create(8, 16 + label_offset + (graphics_v_spacing * 4), 96, 16, "Spin Left:", PGUI_LABEL_JUSTIFY_LEFT, actions_groupbox);
	gamepad_page.spin_left = build_button_combobox(actions_groupbox->w - 8 - box_width, 16 + (graphics_v_spacing * 4), box_width, 20, actions_groupbox, &(padbinds.leftSpin));

	pgui_label_create(8, 16 + label_offset + (graphics_v_spacing * 5), 96, 16, "Spin Right:", PGUI_LABEL_JUSTIFY_LEFT, actions_groupbox);
	gamepad_page.spin_right = build_button_combobox(actions_groupbox->w - 8 - box_width, 16 + (graphics_v_spacing * 5), box_width, 20, actions_groupbox, &(padbinds.rightSpin));

	pgui_label_create(8, 16 + label_offset + (graphics_v_spacing * 6), 96, 16, "Nollie:", PGUI_LABEL_JUSTIFY_LEFT, actions_groupbox);
	gamepad_page.nollie = build_button_combobox(actions_groupbox->w - 8 - box_width, 16 + (graphics_v_spacing * 6), box_width, 20, actions_groupbox, &(padbinds.nollie));

	pgui_label_create(8, 16 + label_offset + (graphics_v_spacing * 7), 96, 16, "Switch:", PGUI_LABEL_JUSTIFY_LEFT, actions_groupbox);
	gamepad_page.switch_revert = build_button_combobox(actions_groupbox->w - 8 - box_width, 16 + (graphics_v_spacing * 7), box_width, 20, actions_groupbox, &(padbinds.switchRevert));

	pgui_label_create(8, 16 + label_offset + (graphics_v_spacing * 8), 96, 16, "Pause:", PGUI_LABEL_JUSTIFY_LEFT, actions_groupbox);
	gamepad_page.pause = build_button_combobox(actions_groupbox->w - 8 - box_width, 16 + (graphics_v_spacing * 8), box_width, 20, actions_groupbox, &(padbinds.menu));

	// skater controls
	pgui_label_create(8, 16 + label_offset, 96, 16, "Forward:", PGUI_LABEL_JUSTIFY_LEFT, skater_groupbox);
	gamepad_page.forward = build_button_combobox(skater_groupbox->w - 8 - box_width, 16, box_width, 20, skater_groupbox, &(padbinds.up));

	pgui_label_create(8, 16 + label_offset + (skater_v_spacing), 96, 16, "Backward:", PGUI_LABEL_JUSTIFY_LEFT, skater_groupbox);
	gamepad_page.backward = build_button_combobox(skater_groupbox->w - 8 - box_width, 16 + (skater_v_spacing), box_width, 20, skater_groupbox, &(padbinds.down));

	pgui_label_create(8, 16 + label_offset + (skater_v_spacing * 2), 96, 16, "Left:", PGUI_LABEL_JUSTIFY_LEFT, skater_groupbox);
	gamepad_page.left = build_button_combobox(skater_groupbox->w - 8 - box_width, 16 + (skater_v_spacing * 2), box_width, 20, skater_groupbox, &(padbinds.left));

	pgui_label_create(8, 16 + label_offset + (skater_v_spacing * 3), 96, 16, "Right:", PGUI_LABEL_JUSTIFY_LEFT, skater_groupbox);
	gamepad_page.right = build_button_combobox(skater_groupbox->w - 8 - box_width, 16 + (skater_v_spacing * 3), box_width, 20, skater_groupbox, &(padbinds.right));

	pgui_label_create(8, 16 + label_offset + (skater_v_spacing * 4), 96, 16, "Move Stick:", PGUI_LABEL_JUSTIFY_LEFT, skater_groupbox);
	gamepad_page.movement_stick = build_stick_combobox(skater_groupbox->w - 8 - box_width, 16 + (skater_v_spacing * 4), box_width, 20, skater_groupbox, &(padbinds.movement));

	// camera controls
	pgui_label_create(8, 16 + label_offset, 96, 16, "Camera Stick:", PGUI_LABEL_JUSTIFY_LEFT, camera_groupbox);
	gamepad_page.camera_stick = build_stick_combobox(skater_groupbox->w - 8 - box_width, 16, box_width, 20, camera_groupbox, &(padbinds.camera));

	pgui_label_create(8, 16 + label_offset + (camera_v_spacing), 96, 16, "View Toggle:", PGUI_LABEL_JUSTIFY_LEFT, camera_groupbox);
	gamepad_page.view_toggle = build_button_combobox(skater_groupbox->w - 8 - box_width, 16 + (camera_v_spacing), box_width, 20, camera_groupbox, &(padbinds.cameraToggle));

	pgui_label_create(8, 16 + label_offset + (camera_v_spacing * 2), 96, 16, "Swivel Lock:", PGUI_LABEL_JUSTIFY_LEFT, camera_groupbox);
	gamepad_page.swivel_lock = build_button_combobox(skater_groupbox->w - 8 - box_width, 16 + (camera_v_spacing * 2), box_width, 20, camera_groupbox, &(padbinds.cameraSwivelLock));
}

void setAllPadBindText() {
	setButtonBindBox(gamepad_page.ollie, padbinds.ollie);
	setButtonBindBox(gamepad_page.grab, padbinds.grab);
	setButtonBindBox(gamepad_page.flip, padbinds.kick);
	setButtonBindBox(gamepad_page.grind, padbinds.grind);
	setButtonBindBox(gamepad_page.spin_left, padbinds.leftSpin);
	setButtonBindBox(gamepad_page.spin_right, padbinds.rightSpin);
	setButtonBindBox(gamepad_page.nollie, padbinds.nollie);
	setButtonBindBox(gamepad_page.switch_revert, padbinds.switchRevert);
	setButtonBindBox(gamepad_page.pause, padbinds.menu);

	setButtonBindBox(gamepad_page.forward, padbinds.up);
	setButtonBindBox(gamepad_page.backward, padbinds.down);
	setButtonBindBox(gamepad_page.left, padbinds.left);
	setButtonBindBox(gamepad_page.right, padbinds.right);
	setStickBindBox(gamepad_page.movement_stick, padbinds.movement);

	setStickBindBox(gamepad_page.camera_stick, padbinds.camera);
	setButtonBindBox(gamepad_page.view_toggle, padbinds.cameraToggle);
	setButtonBindBox(gamepad_page.swivel_lock, padbinds.cameraSwivelLock);
}

struct keyboard_page {
	pgui_control *ollie;
	pgui_control *grab;
	pgui_control *flip;
	pgui_control *grind;
	pgui_control *spin_left;
	pgui_control *spin_right;
	pgui_control *nollie;
	pgui_control *switch_revert;
	pgui_control *pause;

	pgui_control *forward;
	pgui_control *backward;
	pgui_control *left;
	pgui_control *right;

	pgui_control *camera_up;
	pgui_control *camera_down;
	pgui_control *camera_left;
	pgui_control *camera_right;
	pgui_control *view_toggle;
	pgui_control *swivel_lock;
};

struct keyboard_page keyboard_page;

typedef struct {
	SDL_Scancode *target;
	char *name;
} keybind_data;

void do_keybind_select(pgui_control *control, keybind_data *data) {
	doKeyBind(data->name, data->target, control);
}

pgui_control *build_keybind_textbox(int x, int y, int w, int h, pgui_control *parent, char *name, SDL_Scancode *target) {
	pgui_control *result = pgui_textbox_create(x, y, w, h, "", parent);

	keybind_data *data = malloc(sizeof(keybind_data));
	data->name = name;
	data->target = target;

	pgui_textbox_set_on_focus_gained(result, do_keybind_select, data);

	return result;
}

void build_keyboard_page(pgui_control *parent) {
	pgui_control *actions_groupbox = pgui_groupbox_create(8, 8, (parent->w / 2) - 8 - 4, (parent->h) - 8 - 8, "Actions", parent);
	pgui_control *skater_groupbox = pgui_groupbox_create((parent->w / 2) + 4, 8, (parent->w / 2) - 8 - 4, (parent->h / 2) - 8 - 4 - 32, "Skater Controls", parent);
	pgui_control *camera_groupbox = pgui_groupbox_create((parent->w / 2) + 4, (parent->h / 2) + 4 - 32, (parent->w / 2) - 8 - 4, (parent->h / 2) - 8 - 4 + 32, "Camera Controls", parent);

	int label_offset = 4;
	int actions_v_spacing = 39;
	int skater_v_spacing = 32;
	int camera_v_spacing = 32;

	// actions

	pgui_label_create(8, 16 + label_offset, 96, 16, "Ollie:", PGUI_LABEL_JUSTIFY_LEFT, actions_groupbox);
	keyboard_page.ollie = build_keybind_textbox(actions_groupbox->w - 8 - 64, 16, 64, 20, actions_groupbox, "Ollie", &(keybinds.ollie));

	pgui_label_create(8, 16 + label_offset + (actions_v_spacing), 96, 16, "Grab:", PGUI_LABEL_JUSTIFY_LEFT, actions_groupbox);
	keyboard_page.grab = build_keybind_textbox(actions_groupbox->w - 8 - 64, 16 + (actions_v_spacing), 64, 20, actions_groupbox, "Grab", &(keybinds.grab));

	pgui_label_create(8, 16 + label_offset + (actions_v_spacing * 2), 96, 16, "Flip:", PGUI_LABEL_JUSTIFY_LEFT, actions_groupbox);
	keyboard_page.flip = build_keybind_textbox(actions_groupbox->w - 8 - 64, 16 + (actions_v_spacing * 2), 64, 20, actions_groupbox, "Flip", &(keybinds.flip));

	pgui_label_create(8, 16 + label_offset + (actions_v_spacing * 3), 96, 16, "Grind:", PGUI_LABEL_JUSTIFY_LEFT, actions_groupbox);
	keyboard_page.grind = build_keybind_textbox(actions_groupbox->w - 8 - 64, 16 + (actions_v_spacing * 3), 64, 20, actions_groupbox, "Grind", &(keybinds.grind));

	pgui_label_create(8, 16 + label_offset + (actions_v_spacing * 4), 96, 16, "Spin Left:", PGUI_LABEL_JUSTIFY_LEFT, actions_groupbox);
	keyboard_page.spin_left = build_keybind_textbox(actions_groupbox->w - 8 - 64, 16 + (actions_v_spacing * 4), 64, 20, actions_groupbox, "Spin Left", &(keybinds.spinLeft));

	pgui_label_create(8, 16 + label_offset + (actions_v_spacing * 5), 96, 16, "Spin Right:", PGUI_LABEL_JUSTIFY_LEFT, actions_groupbox);
	keyboard_page.spin_right = build_keybind_textbox(actions_groupbox->w - 8 - 64, 16 + (actions_v_spacing * 5), 64, 20, actions_groupbox, "Spin Right", &(keybinds.spinRight));

	pgui_label_create(8, 16 + label_offset + (actions_v_spacing * 6), 96, 16, "Nollie:", PGUI_LABEL_JUSTIFY_LEFT, actions_groupbox);
	keyboard_page.nollie = build_keybind_textbox(actions_groupbox->w - 8 - 64, 16 + (actions_v_spacing * 6), 64, 20, actions_groupbox, "Nollie", &(keybinds.nollie));

	pgui_label_create(8, 16 + label_offset + (actions_v_spacing * 7), 96, 16, "Switch:", PGUI_LABEL_JUSTIFY_LEFT, actions_groupbox);
	keyboard_page.switch_revert = build_keybind_textbox(actions_groupbox->w - 8 - 64, 16 + (actions_v_spacing * 7), 64, 20, actions_groupbox, "Switch", &(keybinds.switchRevert));

	pgui_label_create(8, 16 + label_offset + (actions_v_spacing * 8), 96, 16, "Pause:", PGUI_LABEL_JUSTIFY_LEFT, actions_groupbox);
	keyboard_page.pause = build_keybind_textbox(actions_groupbox->w - 8 - 64, 16 + (actions_v_spacing * 8), 64, 20, actions_groupbox, "Pause", &(keybinds.pause));

	// skater controls
	pgui_label_create(8, 16 + label_offset, 96, 16, "Forward:", PGUI_LABEL_JUSTIFY_LEFT, skater_groupbox);
	keyboard_page.forward = build_keybind_textbox(skater_groupbox->w - 8 - 64, 16, 64, 20, skater_groupbox, "Forward", &(keybinds.forward));

	pgui_label_create(8, 16 + label_offset + (skater_v_spacing), 96, 16, "Backward:", PGUI_LABEL_JUSTIFY_LEFT, skater_groupbox);
	keyboard_page.backward = build_keybind_textbox(skater_groupbox->w - 8 - 64, 16 + (skater_v_spacing), 64, 20, skater_groupbox, "Backward", &(keybinds.backward));

	pgui_label_create(8, 16 + label_offset + (skater_v_spacing * 2), 96, 16, "Left:", PGUI_LABEL_JUSTIFY_LEFT, skater_groupbox);
	keyboard_page.left = build_keybind_textbox(skater_groupbox->w - 8 - 64, 16 + (skater_v_spacing * 2), 64, 20, skater_groupbox, "Left", &(keybinds.left));

	pgui_label_create(8, 16 + label_offset + (skater_v_spacing * 3), 96, 16, "Right:", PGUI_LABEL_JUSTIFY_LEFT, skater_groupbox);
	keyboard_page.right = build_keybind_textbox(skater_groupbox->w - 8 - 64, 16 + (skater_v_spacing * 3), 64, 20, skater_groupbox, "Right", &(keybinds.right));

	// camera controls
	pgui_label_create(8, 16 + label_offset, 96, 16, "Camera Up:", PGUI_LABEL_JUSTIFY_LEFT, camera_groupbox);
	keyboard_page.camera_up = build_keybind_textbox(skater_groupbox->w - 8 - 64, 16, 64, 20, camera_groupbox, "Camera Up", &(keybinds.camUp));

	pgui_label_create(8, 16 + label_offset + (camera_v_spacing), 96, 16, "Camera Down:", PGUI_LABEL_JUSTIFY_LEFT, camera_groupbox);
	keyboard_page.camera_down = build_keybind_textbox(skater_groupbox->w - 8 - 64, 16 + (camera_v_spacing), 64, 20, camera_groupbox, "Camera Down", &(keybinds.camDown));

	pgui_label_create(8, 16 + label_offset + (camera_v_spacing * 2), 96, 16, "Camera Left:", PGUI_LABEL_JUSTIFY_LEFT, camera_groupbox);
	keyboard_page.camera_left = build_keybind_textbox(skater_groupbox->w - 8 - 64, 16 + (camera_v_spacing * 2), 64, 20, camera_groupbox, "Camera Left", &(keybinds.camLeft));

	pgui_label_create(8, 16 + label_offset + (camera_v_spacing * 3), 96, 16, "Camera Right:", PGUI_LABEL_JUSTIFY_LEFT, camera_groupbox);
	keyboard_page.camera_right = build_keybind_textbox(skater_groupbox->w - 8 - 64, 16 + (camera_v_spacing * 3), 64, 20, camera_groupbox, "Camera Right", &(keybinds.camRight));

	pgui_label_create(8, 16 + label_offset + (camera_v_spacing * 4), 96, 16, "View Toggle:", PGUI_LABEL_JUSTIFY_LEFT, camera_groupbox);
	keyboard_page.view_toggle = build_keybind_textbox(skater_groupbox->w - 8 - 64, 16 + (camera_v_spacing * 4), 64, 20, camera_groupbox, "View Toggle", &(keybinds.viewToggle));

	pgui_label_create(8, 16 + label_offset + (camera_v_spacing * 5), 96, 16, "Swivel Lock:", PGUI_LABEL_JUSTIFY_LEFT, camera_groupbox);
	keyboard_page.swivel_lock = build_keybind_textbox(skater_groupbox->w - 8 - 64, 16 + (camera_v_spacing * 5), 64, 20, camera_groupbox, "Swivel Lock", &(keybinds.swivelLock));
}

void setAllBindText() {
	setBindText(keyboard_page.ollie, keybinds.ollie);
	setBindText(keyboard_page.grab, keybinds.grab);
	setBindText(keyboard_page.flip, keybinds.flip);
	setBindText(keyboard_page.grind, keybinds.grind);
	setBindText(keyboard_page.spin_left, keybinds.spinLeft);
	setBindText(keyboard_page.spin_right, keybinds.spinRight);
	setBindText(keyboard_page.nollie, keybinds.nollie);
	setBindText(keyboard_page.switch_revert, keybinds.switchRevert);
	setBindText(keyboard_page.pause, keybinds.pause);

	setBindText(keyboard_page.forward, keybinds.forward);
	setBindText(keyboard_page.backward, keybinds.backward);
	setBindText(keyboard_page.left, keybinds.left);
	setBindText(keyboard_page.right, keybinds.right);

	setBindText(keyboard_page.camera_up, keybinds.camUp);
	setBindText(keyboard_page.camera_down, keybinds.camDown);
	setBindText(keyboard_page.camera_left, keybinds.camLeft);
	setBindText(keyboard_page.camera_right, keybinds.camRight);
	setBindText(keyboard_page.view_toggle, keybinds.viewToggle);
	setBindText(keyboard_page.swivel_lock, keybinds.swivelLock);
}

struct general_page {
	pgui_control *resolution_combobox;
	pgui_control *custom_resolution;
	pgui_control *custom_res_x_label;
	pgui_control *custom_res_x;
	pgui_control *custom_res_y_label;
	pgui_control *custom_res_y;

	pgui_control *windowed;
	pgui_control *borderless;

	pgui_control *shadows;
	pgui_control *particles;
	pgui_control *animating_textures;
	pgui_control *distance_fog;
	pgui_control *low_detail_models;

	pgui_control *play_intro;
	pgui_control *il_mode;
	pgui_control *disable_trick_limit;
};

struct general_page general_page;

void check_custom_resolution(pgui_control *control, int value, void *data) {
	if (value) {
		pgui_label_set_enabled(general_page.custom_res_x_label, 1);
		pgui_textbox_set_enabled(general_page.custom_res_x, 1);
		pgui_label_set_enabled(general_page.custom_res_y_label, 1);
		pgui_textbox_set_enabled(general_page.custom_res_y, 1);
		pgui_combobox_set_enabled(general_page.resolution_combobox, 0);

		char buf[16];
		pgui_textbox_get_text(general_page.custom_res_x, buf, 16);
		settings.resX = atoi(buf);

		pgui_textbox_get_text(general_page.custom_res_y, buf, 16);
		settings.resY = atoi(buf);
	} else {
		pgui_label_set_enabled(general_page.custom_res_x_label, 0);
		pgui_textbox_set_enabled(general_page.custom_res_x, 0);
		pgui_label_set_enabled(general_page.custom_res_y_label, 0);
		pgui_textbox_set_enabled(general_page.custom_res_y, 0);
		pgui_combobox_set_enabled(general_page.resolution_combobox, 1);

		int res_value = pgui_combobox_get_selection(general_page.resolution_combobox);
		if (res_value != 0) {
			settings.resX = displayModeList[res_value - 1].width;
			settings.resY = displayModeList[res_value - 1].height;
		} else {
			settings.resX = 0;
			settings.resY = 0;
		}
	}
}

void set_display_mode(pgui_control *control, int value, void *data) {
	if (value != 0) {
		settings.resX = displayModeList[value - 1].width;
		settings.resY = displayModeList[value - 1].height;
	} else {
		settings.resX = 0;
		settings.resY = 0;
	}
}

void do_custom_resolution_textbox(pgui_control *control, int *target) {
	char buf[16];
	pgui_textbox_get_text(control, buf, 16);
	*target = atoi(buf);
}

void do_setting_checkbox(pgui_control *control, int value, int *target) {
	*target = value;
}

void build_general_page(pgui_control *parent) {
	initResolutionList();

	pgui_control *resolution_groupbox = pgui_groupbox_create(8, 8, parent->w - 8 - 8, (parent->h / 2) - 8 - 4, "Resolution", parent);
	pgui_control *graphics_groupbox = pgui_groupbox_create(8, (parent->h / 2) + 4, (parent->w / 2) - 8 - 4, (parent->h / 2) - 8 - 4, "Graphics", parent);
	pgui_control *misc_groupbox = pgui_groupbox_create((parent->w / 2) + 4, (parent->h / 2) + 4, (parent->w / 2) - 8 - 4, (parent->h / 2) - 8 - 4, "Miscellaneous", parent);

	// resolution options
	general_page.resolution_combobox = pgui_combobox_create(8, 16, 160, 24, displayModeStringList, numDisplayModes + 1, resolution_groupbox);
	general_page.custom_resolution = pgui_checkbox_create(8, 16 + 24, 128, 24, "Use Custom Resolution", resolution_groupbox);

	general_page.custom_res_x_label = pgui_label_create(8 + 8, 16 + (24 * 2) + 4, 48, 24, "Width:", PGUI_LABEL_JUSTIFY_CENTER, resolution_groupbox);
	general_page.custom_res_x = pgui_textbox_create(8 + 8 + 48, 16 + (24 * 2), 48, 20, "", resolution_groupbox);
	general_page.custom_res_y_label = pgui_label_create(8 + 8 + (48 * 2), 16 + (24 * 2) + 4, 48, 24, "Height:", PGUI_LABEL_JUSTIFY_CENTER, resolution_groupbox);
	general_page.custom_res_y = pgui_textbox_create(8 + 8 + (48 * 3), 16 + (24 * 2), 48, 20, "", resolution_groupbox);

	general_page.windowed = pgui_checkbox_create(8, 16 + (24 * 3), 128, 24, "Windowed", resolution_groupbox);
	general_page.borderless = pgui_checkbox_create(8, 16 + (24 * 4), 128, 24, "Borderless", resolution_groupbox);

	// graphics options
	general_page.shadows = pgui_checkbox_create(8, 16, 128, 24, "Shadows", graphics_groupbox);
	general_page.particles = pgui_checkbox_create(8, 16 + 24, 128, 24, "Particles", graphics_groupbox);
	general_page.animating_textures = pgui_checkbox_create(8, 16 + (24 * 2), 128, 24, "Animating Textures", graphics_groupbox);
	general_page.distance_fog = pgui_checkbox_create(8, 16 + (24 * 3), 160, 24, "Distance Fog", graphics_groupbox);
	general_page.low_detail_models = pgui_checkbox_create(8, 16 + (24 * 4), 160, 24, "Low Detail Models", graphics_groupbox);

	// miscellaneous options
	general_page.play_intro = pgui_checkbox_create(8, 16, 128, 24, "Always Play Intro", misc_groupbox);
	general_page.il_mode = pgui_checkbox_create(8, 16 + 24, 128, 24, "IL Mode*", misc_groupbox);
	general_page.disable_trick_limit = pgui_checkbox_create(8, 16 + (24 * 2), 128, 24, "Disable Trick Limit**", misc_groupbox);
	pgui_label_create(8, misc_groupbox->h - 8 - 80, 160, 48, "*For speedrun practice.  Resets goals when a level is retried to practice a single level", PGUI_LABEL_JUSTIFY_LEFT, misc_groupbox);
	pgui_label_create(8, misc_groupbox->h - 8 - 32, 160, 32, "**For scoring. Removes the cap at 251x combo.", PGUI_LABEL_JUSTIFY_LEFT, misc_groupbox);

	pgui_checkbox_set_on_toggle(general_page.windowed, do_setting_checkbox, &(settings.windowed));
	pgui_checkbox_set_on_toggle(general_page.borderless, do_setting_checkbox, &(settings.borderless));

	pgui_checkbox_set_on_toggle(general_page.shadows, do_setting_checkbox, &(settings.shadows));
	pgui_checkbox_set_on_toggle(general_page.particles, do_setting_checkbox, &(settings.particles));
	pgui_checkbox_set_on_toggle(general_page.animating_textures, do_setting_checkbox, &(settings.animating_textures));
	pgui_checkbox_set_on_toggle(general_page.distance_fog, do_setting_checkbox, &(settings.distance_fog));
	pgui_checkbox_set_on_toggle(general_page.low_detail_models, do_setting_checkbox, &(settings.low_detail_models));

	pgui_checkbox_set_on_toggle(general_page.play_intro, do_setting_checkbox, &(settings.play_intro));
	pgui_checkbox_set_on_toggle(general_page.il_mode, do_setting_checkbox, &(settings.il_mode));
	pgui_checkbox_set_on_toggle(general_page.disable_trick_limit, do_setting_checkbox, &(settings.disable_trick_limit));

	pgui_combobox_set_on_select(general_page.resolution_combobox, set_display_mode, NULL);
	pgui_checkbox_set_on_toggle(general_page.custom_resolution, check_custom_resolution, NULL);
	pgui_textbox_set_on_focus_lost(general_page.custom_res_x, do_custom_resolution_textbox, &(settings.resX));
	pgui_textbox_set_on_focus_lost(general_page.custom_res_y, do_custom_resolution_textbox, &(settings.resY));
}

void update_general_page() {
	// find resolution in list
	int foundResolution = 0;
	if (settings.resX == 0 && settings.resY == 0) {
		foundResolution = 1;

		pgui_combobox_set_selection(general_page.resolution_combobox, 0);

		pgui_checkbox_set_checked(general_page.custom_resolution, 0);
		pgui_combobox_set_enabled(general_page.resolution_combobox, 1);
		pgui_label_set_enabled(general_page.custom_res_x_label, 0);
		pgui_textbox_set_enabled(general_page.custom_res_x, 0);
		pgui_label_set_enabled(general_page.custom_res_y_label, 0);
		pgui_textbox_set_enabled(general_page.custom_res_y, 0);
	} else {
		for (int i = 0; i < numDisplayModes; i++) {
			if (displayModeList[i].width == settings.resX && displayModeList[i].height == settings.resY) {
				foundResolution = 1;
				pgui_combobox_set_selection(general_page.resolution_combobox, i + 1);

				pgui_checkbox_set_checked(general_page.custom_resolution, 0);
				pgui_combobox_set_enabled(general_page.resolution_combobox, 1);
				pgui_label_set_enabled(general_page.custom_res_x_label, 0);
				pgui_textbox_set_enabled(general_page.custom_res_x, 0);
				pgui_label_set_enabled(general_page.custom_res_y_label, 0);
				pgui_textbox_set_enabled(general_page.custom_res_y, 0);

				break;
			}
		}
	}

	if (!foundResolution) {
		char buf[16];
		itoa(settings.resX, buf, 10);
		pgui_textbox_set_text(general_page.custom_res_x, buf);
		itoa(settings.resY, buf, 10);
		pgui_textbox_set_text(general_page.custom_res_y, buf);

		pgui_checkbox_set_checked(general_page.custom_resolution, 1);
		pgui_combobox_set_enabled(general_page.resolution_combobox, 0);
		pgui_label_set_enabled(general_page.custom_res_x_label, 1);
		pgui_textbox_set_enabled(general_page.custom_res_x, 1);
		pgui_label_set_enabled(general_page.custom_res_y_label, 1);
		pgui_textbox_set_enabled(general_page.custom_res_y, 1);
	}

	pgui_checkbox_set_checked(general_page.windowed, settings.windowed);
	pgui_checkbox_set_checked(general_page.borderless, settings.borderless);

	pgui_checkbox_set_checked(general_page.shadows, settings.shadows);
	pgui_checkbox_set_checked(general_page.particles, settings.particles);
	pgui_checkbox_set_checked(general_page.animating_textures, settings.animating_textures);
	pgui_checkbox_set_checked(general_page.distance_fog, settings.distance_fog);
	pgui_checkbox_set_checked(general_page.low_detail_models, settings.low_detail_models);

	pgui_checkbox_set_checked(general_page.play_intro, settings.play_intro);
	pgui_checkbox_set_checked(general_page.il_mode, settings.il_mode);
	pgui_checkbox_set_checked(general_page.disable_trick_limit, settings.disable_trick_limit);
}

void callback_ok(pgui_control *control, void *data) {
	saveSettings();
	PostQuitMessage(0);
}

void callback_cancel(pgui_control *control, void *data) {
	PostQuitMessage(0);
}

void callback_default(pgui_control *control, void *data) {
	defaultSettings();
	update_general_page();
	setAllBindText();
	setAllPadBindText();
}

int main(int argc, char **argv) {
	cursedSDLSetup();
	loadSettings();

	pgui_control *window = pgui_window_create(400, 450, "PARTYMOD Configuration");

	pgui_control *restore_default_button = pgui_button_create(8, window->h - 42 + 8, 96, 26, "Restore Defaults", window);
	pgui_control *cancel_button = pgui_button_create(window->w - (88 * 2), window->h - 42 + 8, 80, 26, "Cancel", window);
	pgui_control *ok_button = pgui_button_create(window->w - 88, window->h - 42 + 8, 80, 26, "OK", window);

	pgui_button_set_on_press(restore_default_button, callback_default, NULL);
	pgui_button_set_on_press(cancel_button, callback_cancel, NULL);
	pgui_button_set_on_press(ok_button, callback_ok, NULL);

	char *tab_names[3] = {
		"General",
		"Keyboard",
		"Gamepad",
	};

	pgui_control *tabs = pgui_tabs_create(8, 8, window->w - 16, window->h - 8 - 32 - 8, tab_names, 3, window);

	build_general_page(tabs->children[0]);
	build_keyboard_page(tabs->children[1]);
	build_gamepad_page(tabs->children[2]);

	pgui_control_set_hidden(tabs->children[1], 1);	// bug workaround: controls don't check that the hierarchy is hidden when created, so let's just re-hide it
	pgui_control_set_hidden(tabs->children[2], 1);	// bug workaround: controls don't check that the hierarchy is hidden when created, so let's just re-hide it

	update_general_page();
	setAllBindText();
	setAllPadBindText();
	
	pgui_window_show(window);

	// Run the message loop.

	MSG msg;
	while (GetMessage(&msg, NULL, 0, 0) > 0)
	{
		TranslateMessage(&msg);
		DispatchMessage(&msg);
	}

	return 0;
}