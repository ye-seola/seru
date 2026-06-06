# Seru

## 값 형식

### Size

| 형식 | 예시 | 의미 |
|---|---|---|
| number | `24` | 고정 길이 |
| string | `"24"` | 고정 길이 |
| string | `"100%"` | 부모 기준 비율 |
| string | `"auto"` | 자동 크기 |

### Insets

| 형식 | 예시 | 의미 |
|---|---|---|
| number | `16` | 네 방향 동일 |
| string | `"16"` | 네 방향 동일 |
| string | `"12 16"` | 세로, 가로 |
| string | `"8 12 16 20"` | 위, 오른쪽, 아래, 왼쪽 |

각 값은 `Size` 형식을 따른다.

### Color

| 형식 | 예시 | 의미 |
|---|---|---|
| string | `"red"`, `"green"`, `"blue"` | |
| string | `"#abc"` | rgb `"#aabbcc"` |
| string | `"#ffffff"` | rgb |
| string | `"#ffffffff"` | rgba |

## 공통 스타일

| 인자 | 타입 | 설명 |
|---|---|---|
| `width` | `Size` | 너비 |
| `height` | `Size` | 높이 |
| `max_width` | `Size` | 최대 너비 |
| `max_height` | `Size` | 최대 높이 |
| `min_width` | `Size` | 최소 너비 |
| `min_height` | `Size` | 최소 높이 |
| `margin` | `Insets` | 바깥 여백 |
| `padding` | `Insets` | 안쪽 여백 |
| `border` | number | 테두리 두께 |
| `border_color` | `Color` | 테두리 색상 |
| `background` | `Color` | 배경색 |
| `radius` | number | 모서리 반경 |
| `grow` | number | flex grow |
| `shrink` | number | flex shrink |
| `basis` | number | flex basis |

## Box 계열

`Box`, `Column`, `Row`, `Center`

### 공통 인자

| 인자 | 타입/값 | 설명 |
|---|---|---|
| `gap` | `Size` | 자식 간격 |
| `direction` | `"column"`, `"row"` | 자식 배치 방향 |
| `main` | main align | 주축 정렬 |
| `cross` | cross align | 교차축 정렬 |

### main align

| 값 |
|---|
| `"center"` |
| `"start"` |
| `"end"` |
| `"flex-start"` |
| `"flex-end"` |
| `"between"` |
| `"around"` |
| `"evenly"` |
| `"stretch"` |

### cross align

| 값 |
|---|
| `"start"` |
| `"end"` |
| `"flex-start"` |
| `"flex-end"` |
| `"center"` |
| `"baseline"` |
| `"stretch"` |

### 기본 스타일

| 컴포넌트 | 기본 설정 |
|---|---|
| `Box` | `direction="column"`, `cross="stretch"` |
| `Column` | `direction="column"` |
| `Row` | `direction="row"` |
| `Center` | `direction="column"`, `main="center"`, `cross="center"` |

### Stack

| 인자 | 값 | 설명 |
|---|---|---|
| `place_x` | `"start"`, `"center"`, `"end"` | 가로 배치 위치 |
| `place_y` | `"start"`, `"center"`, `"end"` | 세로 배치 위치 |

## Text

| 인자 | 필수 | 타입/값 | 설명 |
|---|---:|---|---|
| `text` | 예 | string | 출력할 텍스트 |
| `font_family` | 아니오 | string | 폰트 패밀리 |
| `font_weight` | 아니오 | font weight | 폰트 굵기 |
| `font_size` | 아니오 | number | 폰트 크기 |
| `color` | 아니오 | `Color` | 텍스트 색상 (기본 검정) |
| `ellipsis` | 아니오 | bool | 말줄임표 사용 여부 |
| `max_lines` | 아니오 | non-negative integer | 최대 줄 수 |

### font weight

| 값 |
|---|
| `"thin"` |
| `"extra_light"`, `"extralight"` |
| `"light"` |
| `"normal"` |
| `"medium"` |
| `"semibold"` |
| `"bold"` |
| `"extra_bold"`, `"extrabold"` |
| `"black"` |
| `"extra_black"`, `"extrablack"` |


## Image

| 인자 | 필수 | 타입/값 | 설명 |
|---|---:|---|---|
| `src` | 예 | string | 이미지 경로 또는 URL |
| `fit` | 아니오 | image fit | 이미지 맞춤 방식 (기본 Fill) |

### image fit

| 값 | 설명 |
|---|---|
| `"fill"` | 영역에 맞게 늘림 |
| `"cover"` | 영역을 덮도록 채움 |
| `"contain"` | 이미지 전체가 보이도록 맞춤 |

## 특수 컴포넌트

| 컴포넌트 | 설명 |
|---|---|
| `If(condition)[ children ]` | condition이 참일 떄만 children를 보여줌 |
| `For(item in items)[ children ]` | items를 순회하며 item에 넣고 children을 반복 |
| `For(idx, item in items)[ children ]` | items를 순회하며 idx, item에 넣고 children을 반복 |
| `Slot` | 사용자 정의 컴포넌트에서 밖에서 주입된 children을 보여줄 때 사용 |

## 빌트인 함수

| 함수 | 설명 |
|---|---|
| `repeat(value, count)` | value를 count번 반복 |
| `if(condition, true_value, false_value)` | condition이 true면 true_value 아니면 false_value |

## 예시

```seru
component Main():
    Center(width="100%", height="100%", gap=24)[
        Column(
            width=320,
            padding="16 20",
            background="#f5f7fb",
            radius=12,
            gap=12
        )[
            Text(
                text="Hello Seru",
                font_size=24,
                font_weight="bold",
                color="#0d58a3"
            )

            Image(
                src="img1.png",
                width="100%",
                height=180,
                fit="cover",
                radius=8
            )
        ]
    ]
```

```seru
component Child():
    Slot()

component Main():
    Center(width="100%", height="100%", gap=24, background="#f5f7fb")[
        Child()[
            Text(
                text="Hello Seru",
                font_size=24,
                font_weight="bold",
                color="#0d58a3"
            )
        ]
    ]
```
