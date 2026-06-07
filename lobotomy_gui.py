#!/usr/bin/env python3
"""
脑叶公司 弹幕编辑器
"""

import tkinter as tk
from tkinter import ttk, filedialog, colorchooser, messagebox
import json, os, sys, threading, random, subprocess, time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from danmaku_processor import (
    load_fonts, probe_video, make_overlays, process_video,
    parse_color
)


class DanmakuGUI:
    def __init__(self, root):
        self.root = root
        self.root.title("脑叶公司 - 弹幕编辑器")
        self.root.geometry("1280x860")
        self.root.configure(bg="#1a1828")

        self.overlays = []
        self.text_list = []  # [{text, color}, ...]
        self.selected_text_idx = None
        self.video_path = ""
        self.video_info = None
        self.fonts = {}
        self.processing = False

        # 内置字体名
        self.ui_font = "PingFang SC"
        self.mono_font = "Menlo"

        # 视频播放状态
        self.playing = False
        self.play_thread = None
        self.current_frame_img = None  # 保持引用防止GC

        self._build_ui()
        self._load_fonts()
        self._set_color(self.color_var.get())
        self._ed_set_color(self.ed_color_var.get())

    def _load_fonts(self):
        self.fonts = load_fonts()

    def _hex_to_rgb01(self, hex_color):
        r, g, b = int(hex_color[1:3], 16), int(hex_color[3:5], 16), int(hex_color[5:7], 16)
        return [r / 255, g / 255, b / 255]

    def _rgb01_to_hex(self, color):
        r, g, b = (max(0, min(255, int(c * 255))) for c in color[:3])
        return f"#{r:02x}{g:02x}{b:02x}"

    # ════════════════════════ UI ════════════════════════
    def _build_ui(self):
        s = ttk.Style()
        s.theme_use("clam")
        for k, v in [("TFrame", "#1a1828"), ("TLabel", "#1a1828"),
                      ("TLabelframe", "#1a1828"), ("TLabelframe.Label", "#1a1828"),
                      ("TCheckbutton", "#1a1828")]:
            s.configure(k, background=v, foreground="#e8e4f0")
        s.configure("TButton", background="#2a2840", foreground="#e8e4f0")
        s.configure("TEntry", fieldbackground="#2a2840", foreground="#e8e4f0")
        s.configure("TSpinbox", fieldbackground="#2a2840", foreground="#e8e4f0")
        s.configure("Treeview", background="#2a2840", foreground="#e8e4f0",
                     fieldbackground="#2a2840", rowheight=24)
        s.configure("Treeview.Heading", background="#3a3858", foreground="#e8e4f0")
        s.configure("Accent.TButton", background="#b43c3c", foreground="#ffffff")

        main = ttk.PanedWindow(self.root, orient=tk.HORIZONTAL)
        main.pack(fill=tk.BOTH, expand=True, padx=5, pady=5)

        # ═══ 左侧 ═══
        left = ttk.Frame(main, width=540)
        main.add(left, weight=1)

        # 左侧上半部分：内容
        left_top = ttk.Frame(left)
        left_top.pack(fill=tk.BOTH, expand=True)

        # 内容滚动区
        content = tk.Canvas(left_top, bg="#1a1828", highlightthickness=0)
        content.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)
        content_scroll = ttk.Scrollbar(left_top, orient=tk.VERTICAL, command=content.yview)
        content_scroll.pack(side=tk.RIGHT, fill=tk.Y)
        content.configure(yscrollcommand=content_scroll.set)
        self.left_content = ttk.Frame(content)
        content_window = content.create_window((0, 0), window=self.left_content, anchor="nw")

        def _sync_scroll_region(_event=None):
            content.configure(scrollregion=content.bbox("all"))

        def _sync_inner_width(event):
            content.itemconfigure(content_window, width=event.width)

        self.left_content.bind("<Configure>", _sync_scroll_region)
        content.bind("<Configure>", _sync_inner_width)
        self.left_content_canvas = content
        self._content_window = content_window

        def _wheel(event):
            px = self.root.winfo_pointerx()
            py = self.root.winfo_pointery()
            x0 = left.winfo_rootx()
            y0 = left.winfo_rooty()
            x1 = x0 + left.winfo_width()
            y1 = y0 + left.winfo_height()
            if not (x0 <= px <= x1 and y0 <= py <= y1):
                return
            delta = getattr(event, "delta", 0)
            if delta == 0:
                num = getattr(event, "num", None)
                if num == 4:
                    delta = 120
                elif num == 5:
                    delta = -120
            if abs(delta) < 120:
                direction = -1 if delta > 0 else 1
            else:
                direction = int(-delta / 120)
            if direction:
                content.yview_scroll(direction, "units")

        self.root.bind_all("<MouseWheel>", _wheel, add="+")
        self.root.bind_all("<Button-4>", _wheel, add="+")
        self.root.bind_all("<Button-5>", _wheel, add="+")

        self.root.after(100, lambda: main.sashpos(0, 540))

        # 左侧下半部分：导出（固定高度）
        left_bottom = ttk.Frame(left)
        left_bottom.pack(fill=tk.X)

        # ── 视频 ──
        vf = ttk.LabelFrame(self.left_content, text="视频", padding=5)
        vf.pack(fill=tk.X, padx=3, pady=(3,1))
        row = ttk.Frame(vf)
        row.pack(fill=tk.X)
        self.video_label = tk.Label(row, text="  点击选择视频", foreground="#888",
                                     bg="#2a2840", font=(self.ui_font, 10),
                                     cursor="hand2", relief=tk.RAISED, padx=6, pady=4)
        self.video_label.pack(side=tk.LEFT, fill=tk.X, expand=True, padx=(0, 5))
        self.video_label.bind("<Button-1>", lambda e: self._import_video())

        # ── 文字列表 ──
        tl = ttk.LabelFrame(self.left_content, text="文字列表", padding=5)
        tl.pack(fill=tk.X, padx=3, pady=1)

        add_row = ttk.Frame(tl)
        add_row.pack(fill=tk.X, pady=(0, 3))
        ttk.Label(add_row, text="文字:").pack(side=tk.LEFT)
        self.text_var = tk.StringVar()
        self.text_entry = ttk.Entry(add_row, textvariable=self.text_var, width=18)
        self.text_entry.pack(side=tk.LEFT, padx=2)
        self.text_entry.bind("<Return>", lambda e: self._add_text())

        self.color_var = tk.StringVar(value="#b43c3c")
        self.color_btn = tk.Label(add_row, text="#b43c3c", bg="#b43c3c", fg="white",
                                   width=8, padx=4, pady=2, cursor="hand2", font=(self.mono_font, 9))
        self.color_btn.bind("<Button-1>", lambda e: self._pick_color())
        self.color_btn.pack(side=tk.LEFT, padx=2)
        ttk.Button(add_row, text="添加", style="Accent.TButton", command=self._add_text).pack(side=tk.LEFT, padx=4)

        cols = ("text", "color")
        self.text_tree = ttk.Treeview(tl, columns=cols, show="headings", selectmode="browse", height=5)
        self.text_tree.heading("text", text="文字")
        self.text_tree.heading("color", text="颜色")
        self.text_tree.column("text", width=260)
        self.text_tree.column("color", width=80)
        self.text_tree.pack(fill=tk.X)
        self.text_tree.bind("<<TreeviewSelect>>", self._on_text_select)

        btn_row = ttk.Frame(tl)
        btn_row.pack(fill=tk.X, pady=2)
        ttk.Button(btn_row, text="删除", command=self._del_text).pack(side=tk.LEFT, padx=2)
        ttk.Button(btn_row, text="清空", command=self._clear_texts).pack(side=tk.LEFT, padx=2)
        ttk.Button(btn_row, text="改颜色", command=self._change_sel_color).pack(side=tk.LEFT, padx=2)

        # ── 生成参数 ──
        gf = ttk.LabelFrame(self.left_content, text="生成参数", padding=5)
        gf.pack(fill=tk.X, padx=3, pady=1)

        r1 = ttk.Frame(gf)
        r1.pack(fill=tk.X, pady=1)
        ttk.Label(r1, text="密度:").pack(side=tk.LEFT)
        self.density_var = tk.DoubleVar(value=0.45)
        ttk.Spinbox(r1, from_=0.1, to=1.0, increment=0.05, textvariable=self.density_var, width=5).pack(side=tk.LEFT, padx=2)
        ttk.Label(r1, text="最多同时:").pack(side=tk.LEFT, padx=(8, 0))
        self.max_active_var = tk.IntVar(value=4)
        ttk.Spinbox(r1, from_=1, to=20, textvariable=self.max_active_var, width=4).pack(side=tk.LEFT, padx=2)

        r2 = ttk.Frame(gf)
        r2.pack(fill=tk.X, pady=1)
        ttk.Label(r2, text="字号:").pack(side=tk.LEFT)
        self.size_min_var = tk.IntVar(value=18)
        self.size_max_var = tk.IntVar(value=32)
        ttk.Spinbox(r2, from_=8, to=72, textvariable=self.size_min_var, width=4).pack(side=tk.LEFT, padx=2)
        ttk.Label(r2, text="-").pack(side=tk.LEFT)
        ttk.Spinbox(r2, from_=8, to=72, textvariable=self.size_max_var, width=4).pack(side=tk.LEFT, padx=2)
        ttk.Label(r2, text="种子:").pack(side=tk.LEFT, padx=(8, 0))
        self.seed_var = tk.IntVar(value=42)
        ttk.Spinbox(r2, from_=0, to=9999, textvariable=self.seed_var, width=5).pack(side=tk.LEFT, padx=2)

        r3 = ttk.Frame(gf)
        r3.pack(fill=tk.X, pady=1)
        ttk.Label(r3, text="角度:").pack(side=tk.LEFT)
        self.angle_min_var = tk.DoubleVar(value=-14)
        self.angle_max_var = tk.DoubleVar(value=14)
        ttk.Spinbox(r3, from_=-45, to=45, increment=1, textvariable=self.angle_min_var, width=4).pack(side=tk.LEFT, padx=1)
        ttk.Label(r3, text="~").pack(side=tk.LEFT)
        ttk.Spinbox(r3, from_=-45, to=45, increment=1, textvariable=self.angle_max_var, width=4).pack(side=tk.LEFT, padx=1)

        r4 = ttk.Frame(gf)
        r4.pack(fill=tk.X, pady=1)
        ttk.Label(r4, text="打字速度:").pack(side=tk.LEFT)
        self.type_speed_var = tk.DoubleVar(value=0.06)
        ttk.Entry(r4, textvariable=self.type_speed_var, width=6).pack(side=tk.LEFT, padx=2)
        ttk.Label(r4, text="秒/字", foreground="#666").pack(side=tk.LEFT)
        ttk.Label(r4, text="留存:", foreground="#888").pack(side=tk.LEFT, padx=(10, 0))
        self.post_hold_var = tk.DoubleVar(value=1.5)
        ttk.Entry(r4, textvariable=self.post_hold_var, width=6).pack(side=tk.LEFT, padx=2)
        ttk.Label(r4, text="秒", foreground="#666").pack(side=tk.LEFT)

        r5 = ttk.Frame(gf)
        r5.pack(fill=tk.X, pady=1)
        ttk.Label(r5, text="透明度:").pack(side=tk.LEFT)
        self.auto_alpha_var = tk.DoubleVar(value=0.7)
        ttk.Spinbox(r5, from_=0.1, to=1.0, increment=0.05, textvariable=self.auto_alpha_var, width=6).pack(side=tk.LEFT, padx=2)

        gen_row = ttk.Frame(gf)
        gen_row.pack(fill=tk.X, pady=3)
        ttk.Button(gen_row, text="自动生成叠加", style="Accent.TButton", command=self._auto_generate).pack(side=tk.LEFT, padx=2)
        ttk.Button(gen_row, text="全部添加到叠加", command=self._add_all_to_overlays).pack(side=tk.LEFT, padx=2)

        # ── 叠加列表 ──
        of = ttk.LabelFrame(self.left_content, text="叠加列表", padding=5)
        of.pack(fill=tk.X, padx=3, pady=1)

        ocols = ("text", "pos", "time")
        self.over_tree = ttk.Treeview(of, columns=ocols, show="headings", selectmode="browse", height=4)
        self.over_tree.heading("text", text="文字")
        self.over_tree.heading("pos", text="位置")
        self.over_tree.heading("time", text="时间")
        self.over_tree.column("text", width=130)
        self.over_tree.column("pos", width=70)
        self.over_tree.column("time", width=90)
        self.over_tree.pack(fill=tk.X)
        self.over_tree.bind("<<TreeviewSelect>>", self._on_overlay_select)

        obtn = ttk.Frame(of)
        obtn.pack(fill=tk.X, pady=2)
        ttk.Button(obtn, text="删除", command=self._del_overlay).pack(side=tk.LEFT, padx=2)
        ttk.Button(obtn, text="清空", command=self._clear_overlays).pack(side=tk.LEFT, padx=2)

        # ── 手动编辑（精简两行） ──
        ef2 = ttk.LabelFrame(self.left_content, text="手动编辑 (选中叠加后修改)", padding=4)
        ef2.pack(fill=tk.X, padx=3, pady=1)

        # 第1行：文字 + 字号 + 角度
        r1 = ttk.Frame(ef2)
        r1.pack(fill=tk.X, pady=1)
        ttk.Label(r1, text="文字:").pack(side=tk.LEFT)
        self.ed_text = tk.StringVar()
        ttk.Entry(r1, textvariable=self.ed_text, width=20).pack(side=tk.LEFT, padx=2)
        ttk.Label(r1, text="字号:").pack(side=tk.LEFT, padx=(6, 0))
        self.ed_size = tk.IntVar(value=24)
        ttk.Spinbox(r1, from_=8, to=72, textvariable=self.ed_size, width=5).pack(side=tk.LEFT, padx=1)
        ttk.Label(r1, text="角度:").pack(side=tk.LEFT, padx=(6, 0))
        self.ed_angle = tk.DoubleVar(value=0)
        ttk.Spinbox(r1, from_=-45, to=45, increment=0.5, textvariable=self.ed_angle, width=5).pack(side=tk.LEFT, padx=1)

        # 第2行：颜色选择器
        r2 = ttk.Frame(ef2)
        r2.pack(fill=tk.X, pady=1)
        ttk.Label(r2, text="颜色:").pack(side=tk.LEFT)
        self.ed_color_var = tk.StringVar(value="#b43c3c")
        self.ed_color_btn = tk.Label(r2, text="#b43c3c", bg="#b43c3c", fg="white",
                                      width=8, padx=4, pady=2, cursor="hand2", font=(self.mono_font, 9))
        self.ed_color_btn.bind("<Button-1>", lambda e: self._ed_pick_color())
        self.ed_color_btn.pack(side=tk.LEFT, padx=2)

        # 第3行：位置 X Y
        r3 = ttk.Frame(ef2)
        r3.pack(fill=tk.X, pady=1)
        ttk.Label(r3, text="X:").pack(side=tk.LEFT)
        self.ed_x = tk.DoubleVar(value=640)
        ttk.Spinbox(r3, from_=0, to=1920, increment=10, textvariable=self.ed_x, width=6).pack(side=tk.LEFT, padx=1)
        ttk.Label(r3, text="Y:").pack(side=tk.LEFT, padx=(6, 0))
        self.ed_y = tk.DoubleVar(value=360)
        ttk.Spinbox(r3, from_=0, to=1080, increment=10, textvariable=self.ed_y, width=6).pack(side=tk.LEFT, padx=1)

        # 第4行：时间 开始 + 结束 + 留存
        r4 = ttk.Frame(ef2)
        r4.pack(fill=tk.X, pady=1)
        ttk.Label(r4, text="开始:").pack(side=tk.LEFT)
        self.ed_start = tk.DoubleVar(value=0)
        ttk.Spinbox(r4, from_=0, to=300, increment=0.1, textvariable=self.ed_start, width=6).pack(side=tk.LEFT, padx=1)
        ttk.Label(r4, text="结束:").pack(side=tk.LEFT, padx=(4, 0))
        self.ed_end = tk.DoubleVar(value=5)
        ttk.Spinbox(r4, from_=0, to=300, increment=0.1, textvariable=self.ed_end, width=6).pack(side=tk.LEFT, padx=1)
        ttk.Label(r4, text="留存:").pack(side=tk.LEFT, padx=(4, 0))
        self.ed_post_hold = tk.DoubleVar(value=1.5)
        ttk.Spinbox(r4, from_=0, to=10, increment=0.1, textvariable=self.ed_post_hold, width=5).pack(side=tk.LEFT, padx=1)

        # 第5行：打字速度 + 透明度
        r5 = ttk.Frame(ef2)
        r5.pack(fill=tk.X, pady=1)
        ttk.Label(r5, text="打字速度:").pack(side=tk.LEFT)
        self.ed_speed = tk.DoubleVar(value=0.06)
        ttk.Spinbox(r5, from_=0.01, to=0.3, increment=0.01, textvariable=self.ed_speed, width=6).pack(side=tk.LEFT, padx=1)
        ttk.Label(r5, text="透明度:").pack(side=tk.LEFT, padx=(6, 0))
        self.ed_alpha = tk.DoubleVar(value=0.7)
        ttk.Spinbox(r5, from_=0.1, to=1.0, increment=0.05, textvariable=self.ed_alpha, width=5).pack(side=tk.LEFT, padx=1)

        # 按钮行
        btn_row2 = ttk.Frame(ef2)
        btn_row2.pack(fill=tk.X, pady=2)
        ttk.Button(btn_row2, text="添加新叠加", style="Accent.TButton", command=self._manual_add).pack(side=tk.LEFT, padx=2)
        ttk.Button(btn_row2, text="更新选中", command=self._manual_update).pack(side=tk.LEFT, padx=2)

        # ═══ 导出模块 ═══
        export_frame = ttk.LabelFrame(left_bottom, text="导出", padding=8)
        export_frame.pack(fill=tk.X, padx=3, pady=3)

        # 状态行
        self.status_label = ttk.Label(export_frame, text="就绪", foreground="#888")
        self.status_label.pack(fill=tk.X, pady=(0, 5))

        # 按钮行
        btn_frame = ttk.Frame(export_frame)
        btn_frame.pack(fill=tk.X)

        self.export_btn = tk.Button(btn_frame, text="导出视频",
                                     bg="#b43c3c", fg="black",
                                     activebackground="#d44c4c", activeforeground="black",
                                     font=(self.ui_font, 14, "bold"),
                                     height=2, command=self._export_video)
        self.export_btn.pack(fill=tk.X, pady=(0, 5))

        json_row = ttk.Frame(btn_frame)
        json_row.pack(fill=tk.X)
        self.project_save_btn = tk.Button(json_row, text="保存项目", bg="#ccb444", fg="black",
                                         activebackground="#ddc55a", activeforeground="black",
                                         font=(self.ui_font, 11, "bold"), relief=tk.RAISED,
                                         command=self._export_json)
        self.project_save_btn.pack(side=tk.LEFT, padx=(0, 5), fill=tk.X, expand=True)
        self.project_load_btn = tk.Button(json_row, text="加载项目", bg="#4eb0d8", fg="black",
                                         activebackground="#66c0e8", activeforeground="black",
                                         font=(self.ui_font, 11, "bold"), relief=tk.RAISED,
                                         command=self._import_json)
        self.project_load_btn.pack(side=tk.LEFT, fill=tk.X, expand=True)

        preset_row = ttk.Frame(btn_frame)
        preset_row.pack(fill=tk.X, pady=(3, 0))
        ttk.Button(preset_row, text="导出预设 (仅叠加参数)",
                   command=self._export_preset).pack(side=tk.LEFT, fill=tk.X, expand=True)
        ttk.Button(preset_row, text="导入预设",
                   command=self._import_preset).pack(side=tk.LEFT, fill=tk.X, expand=True)

        # ═══ 右侧 - 预览 ═══
        right = ttk.Frame(main, width=800)
        main.add(right, weight=3)

        pf = ttk.LabelFrame(right, text="预览", padding=5)
        pf.pack(fill=tk.BOTH, expand=True, padx=3, pady=3)

        self.canvas = tk.Canvas(pf, bg="#0c0a12", highlightthickness=0)
        self.canvas.pack(fill=tk.BOTH, expand=True)
        self.canvas.bind("<Button-1>", self._on_canvas_click)

        # 播放控制
        pc = ttk.Frame(pf)
        pc.pack(fill=tk.X, pady=3)
        self.play_btn = ttk.Button(pc, text="播放", command=self._toggle_play, width=6)
        self.play_btn.pack(side=tk.LEFT)
        self.preview_var = tk.BooleanVar(value=True)
        ttk.Checkbutton(pc, text="叠加文字", variable=self.preview_var).pack(side=tk.LEFT, padx=(8, 0))
        ttk.Label(pc, text="时间:").pack(side=tk.LEFT, padx=(10, 0))
        self.time_var = tk.DoubleVar(value=0)
        self.time_slider = ttk.Scale(pc, from_=0, to=10, variable=self.time_var,
                                      orient=tk.HORIZONTAL, command=self._on_time_change)
        self.time_slider.pack(side=tk.LEFT, fill=tk.X, expand=True, padx=5)
        self.time_label = ttk.Label(pc, text="0.0s", width=6)
        self.time_label.pack(side=tk.LEFT)

        self.root.bind("<Delete>", lambda e: self._del_overlay())

    # ════════════════════════ 颜色 ════════════════════════
    def _set_color(self, hex_color):
        self.color_var.set(hex_color)
        self._set_button_color(self.color_btn, hex_color, hex_color)

    def _is_dark(self, hex_color):
        """判断颜色是否偏暗"""
        r, g, b = int(hex_color[1:3], 16), int(hex_color[3:5], 16), int(hex_color[5:7], 16)
        return (r * 299 + g * 587 + b * 114) / 1000 < 128

    def _pick_color(self):
        color = colorchooser.askcolor(initialcolor=self.color_var.get())
        if color[1]:
            self._set_color(color[1])

    def _set_button_color(self, button, hex_color, text=None):
        try:
            button.configure(bg=hex_color, activebackground=hex_color,
                             highlightbackground=hex_color, highlightcolor=hex_color,
                             relief=tk.RAISED)
        except:
            pass
        if text is not None:
            button.configure(text=text, fg="white" if self._is_dark(hex_color) else "black")
        else:
            try:
                button.configure(fg="white" if self._is_dark(hex_color) else "black")
            except:
                pass

    def _change_sel_color(self):
        sel = self.text_tree.selection()
        if not sel:
            return
        color = colorchooser.askcolor()
        if color[1]:
            idx = int(sel[0])
            self.text_list[idx]["color"] = color[1]
            self._refresh_text_tree()
            self._on_text_select(None)

    # ════════════════════════ 文字列表 ════════════════════════
    def _add_text(self):
        text = self.text_var.get().strip()
        if not text:
            return
        self.text_list.append({"text": text, "color": self.color_var.get()})
        self._refresh_text_tree()
        self.text_var.set("")
        self.text_entry.focus()
        self.status_label.configure(text=f"已添加: {text}")

    def _del_text(self):
        sel = self.text_tree.selection()
        if not sel:
            return
        idx = int(sel[0])
        if 0 <= idx < len(self.text_list):
            del self.text_list[idx]
            self._refresh_text_tree()

    def _clear_texts(self):
        self.text_list.clear()
        self._refresh_text_tree()

    def _on_text_select(self, event):
        sel = self.text_tree.selection()
        if not sel:
            return
        idx = int(sel[0])
        if 0 <= idx < len(self.text_list):
            item = self.text_list[idx]
            self.text_var.set(item["text"])
            self._set_color(item["color"])

    def _refresh_text_tree(self):
        self.text_tree.delete(*self.text_tree.get_children())
        for i, item in enumerate(self.text_list):
            color_hex = item["color"]
            # 设置 tag 颜色
            tag = f"color_{i}"
            self.text_tree.tag_configure(tag, foreground=color_hex)
            self.text_tree.insert("", tk.END, iid=str(i),
                                  values=(item["text"], color_hex),
                                  tags=(tag,))

    # ════════════════════════ 叠加 ════════════════════════
    def _add_all_to_overlays(self):
        """把文字列表每个文字随机生成一条叠加，遵守 max_active 约束"""
        if not self.text_list:
            messagebox.showwarning("提示", "请先添加文字")
            return
        if not self.video_info:
            messagebox.showwarning("提示", "请先导入视频")
            return
        vw, vh = self.video_info.width, self.video_info.height
        mx, my = vw * 0.15, vh * 0.15
        dur = self.video_info.duration
        max_active = self.max_active_var.get()

        # 先构建 text_specs，用 make_overlays 生成（遵守 max_active）
        text_specs = [(item["text"], tuple(int(c * 255) for c in self._hex_to_rgb01(item["color"])))
                      for item in self.text_list]

        new_overlays = make_overlays(
            text_specs, dur, vw, vh,
            density=self.density_var.get(),
            max_active=max_active,
            seed=self.seed_var.get(),
            size_min=self.size_min_var.get(),
            size_max=self.size_max_var.get(),
            angle_min=self.angle_min_var.get(),
            angle_max=self.angle_max_var.get(),
            type_speed=self.type_speed_var.get(),
            post_hold=self.post_hold_var.get(),
            alpha_max=self.auto_alpha_var.get(),
            fonts=self.fonts,
        )
        self.overlays.extend(new_overlays)
        self._refresh_overlay_tree()
        self._update_preview()
        self.status_label.configure(text=f"已添加 {len(new_overlays)} 条叠加")

    def _del_overlay(self):
        sel = self.over_tree.selection()
        if not sel:
            return
        idx = int(sel[0])
        if 0 <= idx < len(self.overlays):
            del self.overlays[idx]
            self._refresh_overlay_tree()
            self._update_preview()

    def _clear_overlays(self):
        self.overlays.clear()
        self._refresh_overlay_tree()
        self._update_preview()

    # ════════════════════════ 手动编辑 ════════════════════════
    def _ed_pick_color(self):
        color = colorchooser.askcolor(initialcolor=self.ed_color_var.get())
        if color[1]:
            self._ed_set_color(color[1])

    def _ed_set_color(self, hex_color):
        self.ed_color_var.set(hex_color)
        self._set_button_color(self.ed_color_btn, hex_color, hex_color)

    def _manual_add(self):
        """手动添加一条叠加"""
        text = self.ed_text.get().strip()
        if not text:
            messagebox.showwarning("提示", "请输入文字")
            return
        color_hex = self.ed_color_var.get()
        start = self.ed_start.get()
        end = self.ed_end.get()
        if end <= start:
            messagebox.showwarning("提示", "结束时间必须大于开始时间")
            return
        o = {
            "text": text,
            "font_size": self.ed_size.get(),
            "color": self._hex_to_rgb01(color_hex),
            "x": self.ed_x.get(),
            "y": self.ed_y.get(),
            "angle": self.ed_angle.get(),
            "start_time": start,
            "end_time": end,
            "alpha_max": self.ed_alpha.get(),
            "type_speed": self.ed_speed.get(),
            "post_hold": self.ed_post_hold.get(),
        }
        self.overlays.append(o)
        self._refresh_overlay_tree()
        self._update_preview()
        self.status_label.configure(text=f"已添加: {text}")

    def _manual_update(self):
        """更新选中的叠加"""
        sel = self.over_tree.selection()
        if not sel:
            messagebox.showwarning("提示", "请先选中一条叠加")
            return
        idx = int(sel[0])
        if idx < 0 or idx >= len(self.overlays):
            return
        text = self.ed_text.get().strip()
        if not text:
            return
        color_hex = self.ed_color_var.get()
        o = self.overlays[idx]
        o["text"] = text
        o["font_size"] = self.ed_size.get()
        o["color"] = self._hex_to_rgb01(color_hex)
        o["x"] = self.ed_x.get()
        o["y"] = self.ed_y.get()
        o["angle"] = self.ed_angle.get()
        o["start_time"] = self.ed_start.get()
        o["end_time"] = self.ed_end.get()
        o["type_speed"] = self.ed_speed.get()
        o["alpha_max"] = self.ed_alpha.get()
        o["post_hold"] = self.ed_post_hold.get()
        self._refresh_overlay_tree()
        self._update_preview()
        self.status_label.configure(text=f"已更新: {text}")

    def _on_overlay_select(self, event):
        """选中叠加时，填充编辑表单"""
        sel = self.over_tree.selection()
        if not sel:
            return
        idx = int(sel[0])
        if idx < 0 or idx >= len(self.overlays):
            return
        o = self.overlays[idx]
        self.ed_text.set(o["text"])
        self.ed_size.set(o["font_size"])
        self.ed_angle.set(o.get("angle", 0))
        self.ed_x.set(o["x"])
        self.ed_y.set(o["y"])
        self.ed_start.set(o["start_time"])
        self.ed_end.set(o["end_time"])
        self.ed_speed.set(o.get("type_speed", 0.06))
        self.ed_alpha.set(o.get("alpha_max", 0.7))
        self.ed_post_hold.set(o.get("post_hold", 1.5))
        color_hex = self._rgb01_to_hex(o["color"])
        self.ed_color_var.set(color_hex)
        self._set_button_color(self.ed_color_btn, color_hex, "拾色")

    def _refresh_overlay_tree(self):
        self.over_tree.delete(*self.over_tree.get_children())
        for i, o in enumerate(self.overlays):
            self.over_tree.insert("", tk.END, iid=str(i), values=(
                o["text"],
                f"({o['x']:.0f},{o['y']:.0f})",
                f"{o['start_time']:.1f}-{o['end_time']:.1f}",
            ))

    # ════════════════════════ 自动生成 ════════════════════════
    def _auto_generate(self):
        if not self.text_list:
            messagebox.showwarning("提示", "请先添加文字")
            return
        if not self.video_info:
            messagebox.showwarning("提示", "请先导入视频")
            return
        text_specs = [(item["text"], tuple(int(c * 255) for c in self._hex_to_rgb01(item["color"])))
                      for item in self.text_list]
        dur = self.video_info.duration
        w, h = self.video_info.width, self.video_info.height
        self.overlays = make_overlays(
            text_specs, dur, w, h,
            density=self.density_var.get(),
            max_active=self.max_active_var.get(),
            seed=self.seed_var.get(),
            size_min=self.size_min_var.get(),
            size_max=self.size_max_var.get(),
            angle_min=self.angle_min_var.get(),
            angle_max=self.angle_max_var.get(),
            type_speed=self.type_speed_var.get(),
            post_hold=self.post_hold_var.get(),
            alpha_max=self.auto_alpha_var.get(),
            fonts=self.fonts,
        )
        self._refresh_overlay_tree()
        self._update_preview()
        self.time_slider.configure(to=dur)
        self.status_label.configure(text=f"已生成 {len(self.overlays)} 条叠加")

    # ════════════════════════ 视频 ════════════════════════
    def _import_video(self):
        path = filedialog.askopenfilename(
            title="导入视频",
            filetypes=[("视频", "*.mp4 *.mov *.avi *.mkv *.webm"), ("所有", "*.*")]
        )
        if not path:
            return
        try:
            info = probe_video(path)
            self.video_path = path
            self.video_info = info
            self.video_label.configure(
                text=f"  {os.path.basename(path)} ({info.width}x{info.height}, {info.fps:.0f}fps, {info.duration:.1f}s)",
                fg="#e8e4f0")
            self.time_slider.configure(to=info.duration)
            self.time_var.set(0)
            self.status_label.configure(text=f"已加载: {os.path.basename(path)}")
            self._show_frame_at(0)
        except Exception as e:
            messagebox.showerror("错误", f"加载视频失败:\n{e}")

    def _show_frame_at(self, t):
        """从 ffmpeg pipe 读取指定时间的帧并显示"""
        if not self.video_path or not self.video_info:
            return
        tmp = "/tmp/lobotomy_preview.png"
        try:
            subprocess.run([
                "ffmpeg", "-y", "-ss", f"{t:.3f}", "-i", self.video_path,
                "-vframes", "1", "-q:v", "3", tmp
            ], capture_output=True, timeout=5)
            if os.path.exists(tmp):
                self._draw_frame(tmp)
        except:
            pass

    def _draw_frame(self, path):
        """把图片绘制到画布"""
        from PIL import Image, ImageTk
        self.canvas.delete("all")
        cw = self.canvas.winfo_width()
        ch = self.canvas.winfo_height()
        if cw < 10 or ch < 10:
            return

        vw = self.video_info.width if self.video_info else 1280
        vh = self.video_info.height if self.video_info else 720
        scale = min(cw / vw, ch / vh)
        ox = (cw - vw * scale) / 2
        oy = (ch - vh * scale) / 2

        try:
            img = Image.open(path)
            img = img.resize((int(vw * scale), int(vh * scale)), Image.LANCZOS)
            self.current_frame_img = ImageTk.PhotoImage(img)
            self.canvas.create_image(ox, oy, anchor="nw", image=self.current_frame_img)
        except:
            self.canvas.create_rectangle(ox, oy, ox + vw*scale, oy + vh*scale, fill="#0c0a12", outline="")

        # 叠加文字（typewriter 效果）
        if self.preview_var.get():
            t = self.time_var.get()
            for o in self.overlays:
                s = o.get("start_time", 0)
                e = o.get("end_time", 0)
                a = o.get("alpha_max", 0.7)
                if t < s or t > e:
                    continue
                if t < s + 0.4:
                    alpha = min(1.0, (t - s) / 0.4) * a
                elif t > e - 0.3:
                    alpha = min(1.0, (e - t) / 0.3) * a
                else:
                    alpha = a
                if alpha < 0.01:
                    continue

                text = o["text"]
                font_size = o["font_size"]
                type_speed = o.get("type_speed", 0.06)
                elapsed = t - s
                chars_visible = min(len(text), int(elapsed / type_speed))
                if chars_visible <= 0:
                    continue
                visible_text = text[:chars_visible]

                r, g, b = int(o["color"][0]*255), int(o["color"][1]*255), int(o["color"][2]*255)
                r2 = int(r * alpha + 12 * (1 - alpha))
                g2 = int(g * alpha + 10 * (1 - alpha))
                b2 = int(b * alpha + 18 * (1 - alpha))
                color = f"#{r2:02x}{g2:02x}{b2:02x}"

                px = ox + o["x"] * scale
                py = oy + o["y"] * scale
                fs = max(8, int(font_size * scale * 0.8))
                self.canvas.create_text(px, py, text=visible_text, fill=color,
                                        font=(self.ui_font, fs), angle=o.get("angle", 0), anchor="center")

        self.canvas.create_rectangle(ox, oy, ox + vw*scale, oy + vh*scale, outline="#444", width=1)

    # ════════════════════════ 播放 ════════════════════════
    def _toggle_play(self):
        if self.playing:
            self.playing = False
            self.play_btn.configure(text="播放")
        else:
            if not self.video_path:
                return
            self.playing = True
            self.play_btn.configure(text="暂停")
            self.play_thread = threading.Thread(target=self._play_loop, daemon=True)
            self.play_thread.start()

    def _play_loop(self):
        """后台线程：用 ffmpeg pipe 连续读帧播放"""
        if not self.video_path or not self.video_info:
            return
        fps = self.video_info.fps
        dur = self.video_info.duration
        w, h = self.video_info.width, self.video_info.height
        frame_size = w * h * 3

        # 启动 ffmpeg 从当前时间开始读帧
        start_t = self.time_var.get()
        proc = subprocess.Popen([
            "ffmpeg", "-ss", f"{start_t:.3f}", "-i", self.video_path,
            "-f", "rawvideo", "-pix_fmt", "rgb24",
            "-s", f"{w}x{h}", "-"
        ], stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)

        frame_interval = 1.0 / fps
        frame_num = 0

        try:
            while self.playing:
                raw = proc.stdout.read(frame_size)
                if len(raw) < frame_size:
                    break

                t = start_t + frame_num / fps
                if t >= dur:
                    break

                # 更新时间显示
                self.root.after(0, lambda tv=t: self.time_var.set(tv))
                self.root.after(0, lambda tv=t: self.time_label.configure(text=f"{tv:.1f}s"))

                # 直接把 raw RGB 数据绘制到画布
                self.root.after(0, lambda r=raw, t=t: self._draw_raw_frame(r, t))

                frame_num += 1
                # 控制播放速度
                time.sleep(frame_interval * 0.8)  # 略快于实时，避免积压
        finally:
            proc.kill()
            self.root.after(0, lambda: self.play_btn.configure(text="播放"))

    def _draw_raw_frame(self, raw_rgb, t):
        """把 raw RGB 像素直接绘制到画布，带 typewriter 效果"""
        from PIL import Image, ImageTk
        self.canvas.delete("all")
        cw = self.canvas.winfo_width()
        ch = self.canvas.winfo_height()
        if cw < 10 or ch < 10:
            return

        vw, vh = self.video_info.width, self.video_info.height
        scale = min(cw / vw, ch / vh)
        ox = (cw - vw * scale) / 2
        oy = (ch - vh * scale) / 2

        try:
            img = Image.frombytes('RGB', (vw, vh), raw_rgb)
            img = img.resize((int(vw * scale), int(vh * scale)), Image.NEAREST)
            self.current_frame_img = ImageTk.PhotoImage(img)
            self.canvas.create_image(ox, oy, anchor="nw", image=self.current_frame_img)
        except:
            self.canvas.create_rectangle(ox, oy, ox + vw*scale, oy + vh*scale, fill="#0c0a12", outline="")

        # 叠加文字（typewriter 效果）
        if self.preview_var.get():
            for o in self.overlays:
                s = o.get("start_time", 0)
                e = o.get("end_time", 0)
                a = o.get("alpha_max", 0.7)
                if t < s or t > e:
                    continue
                if t < s + 0.4:
                    alpha = min(1.0, (t - s) / 0.4) * a
                elif t > e - 0.3:
                    alpha = min(1.0, (e - t) / 0.3) * a
                else:
                    alpha = a
                if alpha < 0.01:
                    continue

                text = o["text"]
                font_size = o["font_size"]
                type_speed = o.get("type_speed", 0.06)
                elapsed = t - s
                chars_visible = min(len(text), int(elapsed / type_speed))
                if chars_visible <= 0:
                    continue
                visible_text = text[:chars_visible]

                r, g, b = int(o["color"][0]*255), int(o["color"][1]*255), int(o["color"][2]*255)
                # tkinter 不支持 alpha，用颜色亮度模拟透明度
                r2 = int(r * alpha + 12 * (1 - alpha))
                g2 = int(g * alpha + 10 * (1 - alpha))
                b2 = int(b * alpha + 18 * (1 - alpha))
                color = f"#{r2:02x}{g2:02x}{b2:02x}"

                px = ox + o["x"] * scale
                py = oy + o["y"] * scale
                fs = max(8, int(font_size * scale * 0.8))
                self.canvas.create_text(px, py, text=visible_text, fill=color,
                                        font=(self.ui_font, fs), angle=o.get("angle", 0), anchor="center")

        self.canvas.create_rectangle(ox, oy, ox + vw*scale, oy + vh*scale, outline="#444", width=1)

    # ════════════════════════ 画布 ════════════════════════
    def _on_canvas_click(self, event):
        if not self.video_info:
            return
        cw = self.canvas.winfo_width()
        ch = self.canvas.winfo_height()
        vw, vh = self.video_info.width, self.video_info.height
        scale = min(cw / vw, ch / vh)
        ox = (cw - vw * scale) / 2
        oy = (ch - vh * scale) / 2
        vx = (event.x - ox) / scale
        vy = (event.y - oy) / scale
        if 0 <= vx <= vw and 0 <= vy <= vh:
            self.status_label.configure(text=f"位置: ({int(vx)}, {int(vy)})")

    def _on_time_change(self, val):
        t = float(val)
        self.time_label.configure(text=f"{t:.1f}s")
        if not self.playing and self.video_path:
            self._show_frame_at(t)

    def _update_preview(self):
        if self.video_path and not self.playing:
            self._show_frame_at(self.time_var.get())

    # ════════════════════════ 导出 ════════════════════════
    def _export_video(self):
        if not self.video_path:
            messagebox.showwarning("提示", "请先导入视频")
            return
        if not self.overlays:
            messagebox.showwarning("提示", "没有可导出的叠加")
            return
        output_path = filedialog.asksaveasfilename(
            title="保存输出视频", defaultextension=".mp4",
            filetypes=[("MP4", "*.mp4")], initialfile="output.mp4"
        )
        if not output_path:
            return
        self.processing = True
        self.export_btn.configure(text="处理中... 0%")
        self.status_label.configure(text="处理中...")
        self.playing = False
        self.play_btn.configure(text="播放")

        def on_progress(current, total):
            pct = int(current / total * 100) if total > 0 else 0
            self.root.after(0, lambda p=pct: self.export_btn.configure(text=f"处理中... {p}%"))
            self.root.after(0, lambda p=pct: self.status_label.configure(text=f"导出中 {p}%"))

        def run():
            try:
                import traceback
                process_video(self.video_path, output_path, self.overlays, self.fonts, progress_cb=on_progress)
                self.root.after(0, lambda: self.export_btn.configure(text="导出视频"))
                self.root.after(0, lambda: self.status_label.configure(text="完成"))
                self.root.after(0, lambda: messagebox.showinfo("完成", f"已导出:\n{output_path}"))
            except Exception as e:
                tb = traceback.format_exc()
                print(f"[EXPORT ERROR] {tb}")
                self.root.after(0, lambda: self.export_btn.configure(text="导出视频"))
                self.root.after(0, lambda: self.status_label.configure(text=f"错误: {e}"))
                self.root.after(0, lambda: messagebox.showerror("错误", f"{e}\n\n{tb}"))
            finally:
                self.processing = False
        threading.Thread(target=run, daemon=True).start()

    def _export_json(self):
        path = filedialog.asksaveasfilename(
            title="保存JSON", defaultextension=".json",
            filetypes=[("JSON", "*.json")], initialfile="overlays.json"
        )
        if path:
            cleaned = [{k: v for k, v in o.items() if k not in {"get_alpha", "rect"}} for o in self.overlays]
            with open(path, "w", encoding="utf-8") as f:
                json.dump(cleaned, f, ensure_ascii=False, indent=2)
            self.status_label.configure(text="项目已保存")

    def _import_json(self):
        path = filedialog.askopenfilename(title="加载JSON", filetypes=[("JSON", "*.json")])
        if path:
            with open(path, encoding="utf-8") as f:
                self.overlays = json.load(f)
            # 确保每个 overlay 都有 get_alpha 等必要字段
            for o in self.overlays:
                o.setdefault("type_speed", 0.06)
                o.setdefault("post_hold", 1.5)
                o.setdefault("alpha_max", 0.7)
                if isinstance(o.get("color"), str):
                    o["color"] = self._hex_to_rgb01(o["color"])
            self._refresh_overlay_tree()
            self._update_preview()
            self.status_label.configure(text=f"项目已加载 {len(self.overlays)} 条")

    def _export_preset(self):
        """导出预设：文字列表 + 生成参数"""
        if not self.text_list:
            messagebox.showwarning("提示", "没有文字列表可导出")
            return
        path = filedialog.asksaveasfilename(
            title="保存预设", defaultextension=".json",
            filetypes=[("JSON", "*.json")], initialfile="preset.json"
        )
        if not path:
            return
        preset = {
            "text_list": self.text_list,
            "params": {
                "density": self.density_var.get(),
                "max_active": self.max_active_var.get(),
                "size_min": self.size_min_var.get(),
                "size_max": self.size_max_var.get(),
                "angle_min": self.angle_min_var.get(),
                "angle_max": self.angle_max_var.get(),
                "type_speed": self.type_speed_var.get(),
                "post_hold": self.post_hold_var.get(),
                "seed": self.seed_var.get(),
            }
        }
        with open(path, "w", encoding="utf-8") as f:
            json.dump(preset, f, ensure_ascii=False, indent=2)
        self.status_label.configure(text=f"预设已保存: {os.path.basename(path)}")

    def _import_preset(self):
        """导入预设"""
        path = filedialog.askopenfilename(title="加载预设", filetypes=[("JSON", "*.json")])
        if not path:
            return
        with open(path, encoding="utf-8") as f:
            preset = json.load(f)
        # 恢复文字列表
        if "text_list" in preset:
            self.text_list = preset["text_list"]
            for item in self.text_list:
                if isinstance(item.get("color"), list):
                    item["color"] = self._rgb01_to_hex(item["color"])
            self._refresh_text_tree()
        # 恢复参数
        if "params" in preset:
            p = preset["params"]
            self.density_var.set(p.get("density", 0.45))
            self.max_active_var.set(p.get("max_active", 4))
            self.size_min_var.set(p.get("size_min", 18))
            self.size_max_var.set(p.get("size_max", 32))
            self.angle_min_var.set(p.get("angle_min", -14))
            self.angle_max_var.set(p.get("angle_max", 14))
            self.type_speed_var.set(p.get("type_speed", 0.06))
            self.post_hold_var.set(p.get("post_hold", 1.5))
            self.seed_var.set(p.get("seed", 42))
        self.status_label.configure(text=f"预设已加载: {os.path.basename(path)}")


def main():
    root = tk.Tk()
    app = DanmakuGUI(root)
    root.mainloop()

if __name__ == "__main__":
    main()
