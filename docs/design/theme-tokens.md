# Theme tokens: 20 palettes

Generated from the real, committed `src/styles/tokens.css` (task M2-02) — never hand-copied a second time, so this table can never silently drift from what Heimdall actually ships. Every value traces to `$POIROT/Poirot/Sources/Theme/PoirotTheme.swift`: `default`/`solarized`/`highContrast` are its three `ThemePalette` statics; the other 17 are `ColorTheme.palette`'s `.chat(...)` calls (ported from a7t/chat).

`--accent-dim` has no stored Swift token — it is computed (`PoirotTheme.Colors.accentDim`): the theme's own accent at 20% opacity in light, 15% in dark.

**A real, deliberate deviation from the literal transcribed values:** Poirot never actually renders these stored `textPrimary`/`textSecondary` tokens — `PoirotTheme.Colors.textPrimary/textSecondary` instead always return macOS's native, appearance-adaptive label colors (`PoirotTheme.swift:344-349`), which have no cross-platform web equivalent, so Heimdall must use each theme's own stored hex directly (exactly what this task's AC asks for). Because of that, these specific values were *never independently contrast-tested in the real app* — ten of them failed `src/theme/contrast.spec.ts`'s WCAG AA 4.5:1 check once actually used as flat colours. Each was nudged by the minimum hue-preserving lightness change needed to clear 4.5:1 (computed against the worse of `bg-app`/`bg-card`, see `tokens.css`'s own inline comment at each site for the original value and exact ratio): `solarized` dark `textSecondary`, `nightshade` `textSecondary`, `fjord` `textSecondary`, `harbor` `textSecondary`, `parchment` `textSecondary`, `deepSea` `textSecondary`, `daybreak` `textPrimary` and `textSecondary`, `citrus` `textSecondary`, `evergreen` `textSecondary`. Every other value below is Poirot's literal hex, unchanged.

## Default (`default`)

_Appearance-adaptive — follows `prefers-color-scheme`._

| Token | Light | Dark |
|---|---|---|
| `bgApp` | `#F3F4F7` | `#0D0D0F` |
| `bgSidebar` | `#ECEEF3` | `#141416` |
| `bgCard` | `#FFFFFF` | `#1A1A1E` |
| `bgCardHover` | `#F5F7FB` | `#222226` |
| `bgElevated` | `#FCFCFD` | `#222226` |
| `bgCode` | `#EEF1F6` | `#161618` |
| `textPrimary` | `#15161A` | `#F5F5F7` |
| `textSecondary` | `#4E5260` | `#8E8E93` |
| `textTertiary` | `#747A89` | `#636366` |
| `border` | `rgba(0, 0, 0, 0.1)` | `rgba(255, 255, 255, 0.06)` |
| `borderSubtle` | `rgba(0, 0, 0, 0.05)` | `rgba(255, 255, 255, 0.03)` |
| `borderEmphasis` | `rgba(0, 0, 0, 0.16)` | `rgba(255, 255, 255, 0.1)` |
| `green` | `#1F8A36` | `#32D74B` |
| `red` | `#C22A21` | `#FF453A` |
| `blue` | `#005ECF` | `#0A84FF` |
| `orange` | `#B86A00` | `#FF9F0A` |
| `purple` | `#7A3FB5` | `#AF52DE` |
| `teal` | `#0F8F85` | `#30D5C8` |
| `accent` | `#C88422` | `#E8A642` |
| `diffAddBg` | `rgba(31, 138, 54, 0.16)` | `rgba(50, 215, 75, 0.1)` |
| `diffAddText` | `#166A29` | `#32D74B` |
| `diffRemoveBg` | `rgba(194, 42, 33, 0.16)` | `rgba(255, 69, 58, 0.1)` |
| `diffRemoveText` | `#9B221B` | `#FF453A` |
| `accentDim` | `rgba(200, 132, 34, 0.2)` | `rgba(232, 166, 66, 0.15)` |

## Solarized (`solarized`)

_Appearance-adaptive — follows `prefers-color-scheme`._

| Token | Light | Dark |
|---|---|---|
| `bgApp` | `#FDF6E3` | `#002B36` |
| `bgSidebar` | `#EEE8D5` | `#073642` |
| `bgCard` | `#FDF6E3` | `#073642` |
| `bgCardHover` | `#EEE8D5` | `#0A4050` |
| `bgElevated` | `#FDF6E3` | `#0A4050` |
| `bgCode` | `#EEE8D5` | `#002B36` |
| `textPrimary` | `#073642` | `#93A1A1` |
| `textSecondary` | `#586E75` | `#8C9C9E` |
| `textTertiary` | `#93A1A1` | `#657B83` |
| `border` | `rgba(7, 54, 66, 0.12)` | `rgba(131, 148, 150, 0.08)` |
| `borderSubtle` | `rgba(7, 54, 66, 0.06)` | `rgba(131, 148, 150, 0.04)` |
| `borderEmphasis` | `rgba(7, 54, 66, 0.2)` | `rgba(131, 148, 150, 0.14)` |
| `green` | `#859900` | `#859900` |
| `red` | `#DC322F` | `#DC322F` |
| `blue` | `#268BD2` | `#268BD2` |
| `orange` | `#CB4B16` | `#CB4B16` |
| `purple` | `#6C71C4` | `#6C71C4` |
| `teal` | `#2AA198` | `#2AA198` |
| `accent` | `#268BD2` | `#2AA198` |
| `diffAddBg` | `rgba(133, 153, 0, 0.16)` | `rgba(133, 153, 0, 0.12)` |
| `diffAddText` | `#859900` | `#859900` |
| `diffRemoveBg` | `rgba(220, 50, 47, 0.16)` | `rgba(220, 50, 47, 0.12)` |
| `diffRemoveText` | `#DC322F` | `#DC322F` |
| `accentDim` | `rgba(38, 139, 210, 0.2)` | `rgba(42, 161, 152, 0.15)` |

## High Contrast (`highContrast`)

_Appearance-adaptive — follows `prefers-color-scheme`._

| Token | Light | Dark |
|---|---|---|
| `bgApp` | `#FFFFFF` | `#000000` |
| `bgSidebar` | `#F0F0F0` | `#0A0A0A` |
| `bgCard` | `#FFFFFF` | `#111111` |
| `bgCardHover` | `#E8E8E8` | `#1A1A1A` |
| `bgElevated` | `#FFFFFF` | `#1A1A1A` |
| `bgCode` | `#F0F0F0` | `#0A0A0A` |
| `textPrimary` | `#000000` | `#FFFFFF` |
| `textSecondary` | `#222222` | `#DDDDDD` |
| `textTertiary` | `#444444` | `#AAAAAA` |
| `border` | `rgba(0, 0, 0, 0.2)` | `rgba(255, 255, 255, 0.15)` |
| `borderSubtle` | `rgba(0, 0, 0, 0.12)` | `rgba(255, 255, 255, 0.08)` |
| `borderEmphasis` | `rgba(0, 0, 0, 0.3)` | `rgba(255, 255, 255, 0.25)` |
| `green` | `#007A1B` | `#3DE858` |
| `red` | `#CC0000` | `#FF4444` |
| `blue` | `#0050CC` | `#4499FF` |
| `orange` | `#CC6600` | `#FFAA22` |
| `purple` | `#6633CC` | `#BB77FF` |
| `teal` | `#007777` | `#33DDCC` |
| `accent` | `#0050CC` | `#4499FF` |
| `diffAddBg` | `rgba(0, 122, 27, 0.2)` | `rgba(61, 232, 88, 0.15)` |
| `diffAddText` | `#007A1B` | `#3DE858` |
| `diffRemoveBg` | `rgba(204, 0, 0, 0.2)` | `rgba(255, 68, 68, 0.15)` |
| `diffRemoveText` | `#CC0000` | `#FF4444` |
| `accentDim` | `rgba(0, 80, 204, 0.2)` | `rgba(68, 153, 255, 0.15)` |

## Charcoal (`charcoal`)

_Fixed appearance: color-scheme `undefined`._

| Token | Value |
|---|---|
| `bgApp` | `#101012` |
| `bgSidebar` | `#19191C` |
| `bgCard` | `#19191C` |
| `bgCardHover` | `#212126` |
| `bgElevated` | `#212126` |
| `bgCode` | `#101012` |
| `textPrimary` | `#ECECEC` |
| `textSecondary` | `#9B9BA2` |
| `textTertiary` | `rgba(155, 155, 162, 0.7)` |
| `border` | `#2B2B30` |
| `borderSubtle` | `rgba(43, 43, 48, 0.5)` |
| `borderEmphasis` | `#2B2B30` |
| `green` | `#32D74B` |
| `red` | `#FF453A` |
| `blue` | `#0A84FF` |
| `orange` | `#FF9F0A` |
| `purple` | `#AF52DE` |
| `teal` | `#2D9B9B` |
| `accent` | `#2D9B9B` |
| `diffAddBg` | `rgba(50, 215, 75, 0.12)` |
| `diffAddText` | `#32D74B` |
| `diffRemoveBg` | `rgba(255, 69, 58, 0.12)` |
| `diffRemoveText` | `#FF453A` |
| `accentDim` | `rgba(45, 155, 155, 0.15)` |

## Teal Night (`tealNight`)

_Fixed appearance: color-scheme `undefined`._

| Token | Value |
|---|---|
| `bgApp` | `#0B3838` |
| `bgSidebar` | `#0F4444` |
| `bgCard` | `#0F4444` |
| `bgCardHover` | `#135050` |
| `bgElevated` | `#135050` |
| `bgCode` | `#0B3838` |
| `textPrimary` | `#D7ECEC` |
| `textSecondary` | `#7FCDCD` |
| `textTertiary` | `rgba(127, 205, 205, 0.7)` |
| `border` | `#1F6B6B` |
| `borderSubtle` | `rgba(31, 107, 107, 0.5)` |
| `borderEmphasis` | `#1F6B6B` |
| `green` | `#32D74B` |
| `red` | `#FF453A` |
| `blue` | `#0A84FF` |
| `orange` | `#FF9F0A` |
| `purple` | `#AF52DE` |
| `teal` | `#7FCDCD` |
| `accent` | `#7FCDCD` |
| `diffAddBg` | `rgba(50, 215, 75, 0.12)` |
| `diffAddText` | `#32D74B` |
| `diffRemoveBg` | `rgba(255, 69, 58, 0.12)` |
| `diffRemoveText` | `#FF453A` |
| `accentDim` | `rgba(127, 205, 205, 0.15)` |

## White (`white`)

_Fixed appearance: color-scheme `undefined`._

| Token | Value |
|---|---|
| `bgApp` | `#FFFFFF` |
| `bgSidebar` | `#F5F4F1` |
| `bgCard` | `#F5F4F1` |
| `bgCardHover` | `#ECEBE6` |
| `bgElevated` | `#ECEBE6` |
| `bgCode` | `#ECEBE6` |
| `textPrimary` | `#0A0F0F` |
| `textSecondary` | `#4A5454` |
| `textTertiary` | `rgba(74, 84, 84, 0.7)` |
| `border` | `#E3E1DA` |
| `borderSubtle` | `rgba(227, 225, 218, 0.5)` |
| `borderEmphasis` | `#E3E1DA` |
| `green` | `#1F8A36` |
| `red` | `#C22A21` |
| `blue` | `#005ECF` |
| `orange` | `#B86A00` |
| `purple` | `#7A3FB5` |
| `teal` | `#1F6B6B` |
| `accent` | `#1F6B6B` |
| `diffAddBg` | `rgba(31, 138, 54, 0.16)` |
| `diffAddText` | `#1F8A36` |
| `diffRemoveBg` | `rgba(194, 42, 33, 0.16)` |
| `diffRemoveText` | `#C22A21` |
| `accentDim` | `rgba(31, 107, 107, 0.2)` |

## Pastel (`pastel`)

_Fixed appearance: color-scheme `undefined`._

| Token | Value |
|---|---|
| `bgApp` | `#F7F1E8` |
| `bgSidebar` | `#EFE6D3` |
| `bgCard` | `#EFE6D3` |
| `bgCardHover` | `#E7DCC4` |
| `bgElevated` | `#E7DCC4` |
| `bgCode` | `#E7DCC4` |
| `textPrimary` | `#0A0F0F` |
| `textSecondary` | `#4A5454` |
| `textTertiary` | `rgba(74, 84, 84, 0.7)` |
| `border` | `#DCCFB4` |
| `borderSubtle` | `rgba(220, 207, 180, 0.5)` |
| `borderEmphasis` | `#DCCFB4` |
| `green` | `#1F8A36` |
| `red` | `#C22A21` |
| `blue` | `#005ECF` |
| `orange` | `#B86A00` |
| `purple` | `#7A3FB5` |
| `teal` | `#1F6B6B` |
| `accent` | `#1F6B6B` |
| `diffAddBg` | `rgba(31, 138, 54, 0.16)` |
| `diffAddText` | `#1F8A36` |
| `diffRemoveBg` | `rgba(194, 42, 33, 0.16)` |
| `diffRemoveText` | `#C22A21` |
| `accentDim` | `rgba(31, 107, 107, 0.2)` |

## Green (`greenTheme`)

_Fixed appearance: color-scheme `undefined`._

| Token | Value |
|---|---|
| `bgApp` | `#EEF3EF` |
| `bgSidebar` | `#E0EBE2` |
| `bgCard` | `#E0EBE2` |
| `bgCardHover` | `#D3E2D6` |
| `bgElevated` | `#D3E2D6` |
| `bgCode` | `#D3E2D6` |
| `textPrimary` | `#0A1F17` |
| `textSecondary` | `#45615A` |
| `textTertiary` | `rgba(69, 97, 90, 0.7)` |
| `border` | `#C5D8C9` |
| `borderSubtle` | `rgba(197, 216, 201, 0.5)` |
| `borderEmphasis` | `#C5D8C9` |
| `green` | `#1F8A36` |
| `red` | `#C22A21` |
| `blue` | `#005ECF` |
| `orange` | `#B86A00` |
| `purple` | `#7A3FB5` |
| `teal` | `#1F6B6B` |
| `accent` | `#1F6B6B` |
| `diffAddBg` | `rgba(31, 138, 54, 0.16)` |
| `diffAddText` | `#1F8A36` |
| `diffRemoveBg` | `rgba(194, 42, 33, 0.16)` |
| `diffRemoveText` | `#C22A21` |
| `accentDim` | `rgba(31, 107, 107, 0.2)` |

## Nightshade (`nightshade`)

_Fixed appearance: color-scheme `undefined`._

| Token | Value |
|---|---|
| `bgApp` | `#282A36` |
| `bgSidebar` | `#2F3140` |
| `bgCard` | `#2F3140` |
| `bgCardHover` | `#383A4A` |
| `bgElevated` | `#383A4A` |
| `bgCode` | `#282A36` |
| `textPrimary` | `#F8F8F2` |
| `textSecondary` | `#9599AB` |
| `textTertiary` | `rgba(139, 143, 163, 0.7)` |
| `border` | `#44475A` |
| `borderSubtle` | `rgba(68, 71, 90, 0.5)` |
| `borderEmphasis` | `#44475A` |
| `green` | `#32D74B` |
| `red` | `#FF453A` |
| `blue` | `#0A84FF` |
| `orange` | `#FF9F0A` |
| `purple` | `#AF52DE` |
| `teal` | `#BD93F9` |
| `accent` | `#BD93F9` |
| `diffAddBg` | `rgba(50, 215, 75, 0.12)` |
| `diffAddText` | `#32D74B` |
| `diffRemoveBg` | `rgba(255, 69, 58, 0.12)` |
| `diffRemoveText` | `#FF453A` |
| `accentDim` | `rgba(189, 147, 249, 0.15)` |

## Fjord (`fjord`)

_Fixed appearance: color-scheme `undefined`._

| Token | Value |
|---|---|
| `bgApp` | `#2E3440` |
| `bgSidebar` | `#353C4A` |
| `bgCard` | `#353C4A` |
| `bgCardHover` | `#3B4252` |
| `bgElevated` | `#3B4252` |
| `bgCode` | `#2E3440` |
| `textPrimary` | `#ECEFF4` |
| `textSecondary` | `#9EA7B8` |
| `textTertiary` | `rgba(136, 147, 168, 0.7)` |
| `border` | `#434C5E` |
| `borderSubtle` | `rgba(67, 76, 94, 0.5)` |
| `borderEmphasis` | `#434C5E` |
| `green` | `#32D74B` |
| `red` | `#FF453A` |
| `blue` | `#0A84FF` |
| `orange` | `#FF9F0A` |
| `purple` | `#AF52DE` |
| `teal` | `#88C0D0` |
| `accent` | `#88C0D0` |
| `diffAddBg` | `rgba(50, 215, 75, 0.12)` |
| `diffAddText` | `#32D74B` |
| `diffRemoveBg` | `rgba(255, 69, 58, 0.12)` |
| `diffRemoveText` | `#FF453A` |
| `accentDim` | `rgba(136, 192, 208, 0.15)` |

## Fjord Light (`fjordLight`)

_Fixed appearance: color-scheme `undefined`._

| Token | Value |
|---|---|
| `bgApp` | `#ECEFF4` |
| `bgSidebar` | `#E5E9F0` |
| `bgCard` | `#E5E9F0` |
| `bgCardHover` | `#D8DEE9` |
| `bgElevated` | `#D8DEE9` |
| `bgCode` | `#D8DEE9` |
| `textPrimary` | `#2E3440` |
| `textSecondary` | `#4C566A` |
| `textTertiary` | `rgba(76, 86, 106, 0.7)` |
| `border` | `#C8D0DE` |
| `borderSubtle` | `rgba(200, 208, 222, 0.5)` |
| `borderEmphasis` | `#C8D0DE` |
| `green` | `#1F8A36` |
| `red` | `#C22A21` |
| `blue` | `#005ECF` |
| `orange` | `#B86A00` |
| `purple` | `#7A3FB5` |
| `teal` | `#5E81AC` |
| `accent` | `#5E81AC` |
| `diffAddBg` | `rgba(31, 138, 54, 0.16)` |
| `diffAddText` | `#1F8A36` |
| `diffRemoveBg` | `rgba(194, 42, 33, 0.16)` |
| `diffRemoveText` | `#C22A21` |
| `accentDim` | `rgba(94, 129, 172, 0.2)` |

## Harbor (`harbor`)

_Fixed appearance: color-scheme `undefined`._

| Token | Value |
|---|---|
| `bgApp` | `#1A1B26` |
| `bgSidebar` | `#1F2335` |
| `bgCard` | `#1F2335` |
| `bgCardHover` | `#24283B` |
| `bgElevated` | `#24283B` |
| `bgCode` | `#1A1B26` |
| `textPrimary` | `#C0CAF5` |
| `textSecondary` | `#8689A3` |
| `textTertiary` | `rgba(120, 124, 153, 0.7)` |
| `border` | `#2F334D` |
| `borderSubtle` | `rgba(47, 51, 77, 0.5)` |
| `borderEmphasis` | `#2F334D` |
| `green` | `#32D74B` |
| `red` | `#FF453A` |
| `blue` | `#0A84FF` |
| `orange` | `#FF9F0A` |
| `purple` | `#AF52DE` |
| `teal` | `#7AA2F7` |
| `accent` | `#7AA2F7` |
| `diffAddBg` | `rgba(50, 215, 75, 0.12)` |
| `diffAddText` | `#32D74B` |
| `diffRemoveBg` | `rgba(255, 69, 58, 0.12)` |
| `diffRemoveText` | `#FF453A` |
| `accentDim` | `rgba(122, 162, 247, 0.15)` |

## Mocha (`mocha`)

_Fixed appearance: color-scheme `undefined`._

| Token | Value |
|---|---|
| `bgApp` | `#1E1E2E` |
| `bgSidebar` | `#242436` |
| `bgCard` | `#242436` |
| `bgCardHover` | `#313244` |
| `bgElevated` | `#313244` |
| `bgCode` | `#1E1E2E` |
| `textPrimary` | `#CDD6F4` |
| `textSecondary` | `#9399B2` |
| `textTertiary` | `rgba(147, 153, 178, 0.7)` |
| `border` | `#45475A` |
| `borderSubtle` | `rgba(69, 71, 90, 0.5)` |
| `borderEmphasis` | `#45475A` |
| `green` | `#32D74B` |
| `red` | `#FF453A` |
| `blue` | `#0A84FF` |
| `orange` | `#FF9F0A` |
| `purple` | `#AF52DE` |
| `teal` | `#CBA6F7` |
| `accent` | `#CBA6F7` |
| `diffAddBg` | `rgba(50, 215, 75, 0.12)` |
| `diffAddText` | `#32D74B` |
| `diffRemoveBg` | `rgba(255, 69, 58, 0.12)` |
| `diffRemoveText` | `#FF453A` |
| `accentDim` | `rgba(203, 166, 247, 0.15)` |

## Rosewood (`rosewood`)

_Fixed appearance: color-scheme `undefined`._

| Token | Value |
|---|---|
| `bgApp` | `#191724` |
| `bgSidebar` | `#1F1D2E` |
| `bgCard` | `#1F1D2E` |
| `bgCardHover` | `#26233A` |
| `bgElevated` | `#26233A` |
| `bgCode` | `#191724` |
| `textPrimary` | `#E0DEF4` |
| `textSecondary` | `#908CAA` |
| `textTertiary` | `rgba(144, 140, 170, 0.7)` |
| `border` | `#403D52` |
| `borderSubtle` | `rgba(64, 61, 82, 0.5)` |
| `borderEmphasis` | `#403D52` |
| `green` | `#32D74B` |
| `red` | `#FF453A` |
| `blue` | `#0A84FF` |
| `orange` | `#FF9F0A` |
| `purple` | `#AF52DE` |
| `teal` | `#EBBCBA` |
| `accent` | `#EBBCBA` |
| `diffAddBg` | `rgba(50, 215, 75, 0.12)` |
| `diffAddText` | `#32D74B` |
| `diffRemoveBg` | `rgba(255, 69, 58, 0.12)` |
| `diffRemoveText` | `#FF453A` |
| `accentDim` | `rgba(235, 188, 186, 0.15)` |

## Ember (`ember`)

_Fixed appearance: color-scheme `undefined`._

| Token | Value |
|---|---|
| `bgApp` | `#282828` |
| `bgSidebar` | `#32302F` |
| `bgCard` | `#32302F` |
| `bgCardHover` | `#3C3836` |
| `bgElevated` | `#3C3836` |
| `bgCode` | `#282828` |
| `textPrimary` | `#EBDBB2` |
| `textSecondary` | `#A89984` |
| `textTertiary` | `rgba(168, 153, 132, 0.7)` |
| `border` | `#504945` |
| `borderSubtle` | `rgba(80, 73, 69, 0.5)` |
| `borderEmphasis` | `#504945` |
| `green` | `#32D74B` |
| `red` | `#FF453A` |
| `blue` | `#0A84FF` |
| `orange` | `#FF9F0A` |
| `purple` | `#AF52DE` |
| `teal` | `#FABD2F` |
| `accent` | `#FABD2F` |
| `diffAddBg` | `rgba(50, 215, 75, 0.12)` |
| `diffAddText` | `#32D74B` |
| `diffRemoveBg` | `rgba(255, 69, 58, 0.12)` |
| `diffRemoveText` | `#FF453A` |
| `accentDim` | `rgba(250, 189, 47, 0.15)` |

## Parchment (`parchment`)

_Fixed appearance: color-scheme `undefined`._

| Token | Value |
|---|---|
| `bgApp` | `#FBF1C7` |
| `bgSidebar` | `#F2E5BC` |
| `bgCard` | `#F2E5BC` |
| `bgCardHover` | `#EBDBB2` |
| `bgElevated` | `#EBDBB2` |
| `bgCode` | `#EBDBB2` |
| `textPrimary` | `#3C3836` |
| `textSecondary` | `#70645A` |
| `textTertiary` | `rgba(124, 111, 100, 0.7)` |
| `border` | `#D5C4A1` |
| `borderSubtle` | `rgba(213, 196, 161, 0.5)` |
| `borderEmphasis` | `#D5C4A1` |
| `green` | `#1F8A36` |
| `red` | `#C22A21` |
| `blue` | `#005ECF` |
| `orange` | `#B86A00` |
| `purple` | `#7A3FB5` |
| `teal` | `#B57614` |
| `accent` | `#B57614` |
| `diffAddBg` | `rgba(31, 138, 54, 0.16)` |
| `diffAddText` | `#1F8A36` |
| `diffRemoveBg` | `rgba(194, 42, 33, 0.16)` |
| `diffRemoveText` | `#C22A21` |
| `accentDim` | `rgba(181, 118, 20, 0.2)` |

## Deep Sea (`deepSea`)

_Fixed appearance: color-scheme `undefined`._

| Token | Value |
|---|---|
| `bgApp` | `#002B36` |
| `bgSidebar` | `#073642` |
| `bgCard` | `#073642` |
| `bgCardHover` | `#0A4250` |
| `bgElevated` | `#0A4250` |
| `bgCode` | `#002B36` |
| `textPrimary` | `#93A1A1` |
| `textSecondary` | `#889CA3` |
| `textTertiary` | `rgba(101, 123, 131, 0.7)` |
| `border` | `#0F4D5C` |
| `borderSubtle` | `rgba(15, 77, 92, 0.5)` |
| `borderEmphasis` | `#0F4D5C` |
| `green` | `#32D74B` |
| `red` | `#FF453A` |
| `blue` | `#0A84FF` |
| `orange` | `#FF9F0A` |
| `purple` | `#AF52DE` |
| `teal` | `#2AA198` |
| `accent` | `#2AA198` |
| `diffAddBg` | `rgba(50, 215, 75, 0.12)` |
| `diffAddText` | `#32D74B` |
| `diffRemoveBg` | `rgba(255, 69, 58, 0.12)` |
| `diffRemoveText` | `#FF453A` |
| `accentDim` | `rgba(42, 161, 152, 0.15)` |

## Daybreak (`daybreak`)

_Fixed appearance: color-scheme `undefined`._

| Token | Value |
|---|---|
| `bgApp` | `#FDF6E3` |
| `bgSidebar` | `#EEE8D5` |
| `bgCard` | `#EEE8D5` |
| `bgCardHover` | `#E4DDC8` |
| `bgElevated` | `#E4DDC8` |
| `bgCode` | `#E4DDC8` |
| `textPrimary` | `#566C72` |
| `textSecondary` | `#5D6B6B` |
| `textTertiary` | `rgba(147, 161, 161, 0.7)` |
| `border` | `#D6CFB8` |
| `borderSubtle` | `rgba(214, 207, 184, 0.5)` |
| `borderEmphasis` | `#D6CFB8` |
| `green` | `#1F8A36` |
| `red` | `#C22A21` |
| `blue` | `#005ECF` |
| `orange` | `#B86A00` |
| `purple` | `#7A3FB5` |
| `teal` | `#268BD2` |
| `accent` | `#268BD2` |
| `diffAddBg` | `rgba(31, 138, 54, 0.16)` |
| `diffAddText` | `#1F8A36` |
| `diffRemoveBg` | `rgba(194, 42, 33, 0.16)` |
| `diffRemoveText` | `#C22A21` |
| `accentDim` | `rgba(38, 139, 210, 0.2)` |

## Citrus (`citrus`)

_Fixed appearance: color-scheme `undefined`._

| Token | Value |
|---|---|
| `bgApp` | `#272822` |
| `bgSidebar` | `#2F302A` |
| `bgCard` | `#2F302A` |
| `bgCardHover` | `#3A3B32` |
| `bgElevated` | `#3A3B32` |
| `bgCode` | `#272822` |
| `textPrimary` | `#F8F8F2` |
| `textSecondary` | `#9B9784` |
| `textTertiary` | `rgba(154, 150, 130, 0.7)` |
| `border` | `#49483E` |
| `borderSubtle` | `rgba(73, 72, 62, 0.5)` |
| `borderEmphasis` | `#49483E` |
| `green` | `#32D74B` |
| `red` | `#FF453A` |
| `blue` | `#0A84FF` |
| `orange` | `#FF9F0A` |
| `purple` | `#AF52DE` |
| `teal` | `#A6E22E` |
| `accent` | `#A6E22E` |
| `diffAddBg` | `rgba(50, 215, 75, 0.12)` |
| `diffAddText` | `#32D74B` |
| `diffRemoveBg` | `rgba(255, 69, 58, 0.12)` |
| `diffRemoveText` | `#FF453A` |
| `accentDim` | `rgba(166, 226, 46, 0.15)` |

## Evergreen (`evergreen`)

_Fixed appearance: color-scheme `undefined`._

| Token | Value |
|---|---|
| `bgApp` | `#2D353B` |
| `bgSidebar` | `#343F44` |
| `bgCard` | `#343F44` |
| `bgCardHover` | `#3D484D` |
| `bgElevated` | `#3D484D` |
| `bgCode` | `#2D353B` |
| `textPrimary` | `#D3C6AA` |
| `textSecondary` | `#A0ABA4` |
| `textTertiary` | `rgba(133, 146, 137, 0.7)` |
| `border` | `#4A555B` |
| `borderSubtle` | `rgba(74, 85, 91, 0.5)` |
| `borderEmphasis` | `#4A555B` |
| `green` | `#32D74B` |
| `red` | `#FF453A` |
| `blue` | `#0A84FF` |
| `orange` | `#FF9F0A` |
| `purple` | `#AF52DE` |
| `teal` | `#A7C080` |
| `accent` | `#A7C080` |
| `diffAddBg` | `rgba(50, 215, 75, 0.12)` |
| `diffAddText` | `#32D74B` |
| `diffRemoveBg` | `rgba(255, 69, 58, 0.12)` |
| `diffRemoveText` | `#FF453A` |
| `accentDim` | `rgba(167, 192, 128, 0.15)` |

