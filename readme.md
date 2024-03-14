# mdbook-typst-piggsoft

## 是什么

mdbook-typst-piggsoft是一个mdbook的output链，主要是将markdown文件导出为pdf、svg、png。

### 主要参考

主要感谢如下作品，部分是仿造进行实现

也解决了无法导出图片的问题

- [mdbook-typst-pdf](https://github.com/KaiserY/mdbook-typst-pdf)
- [mdbook-typst](https://github.com/LegNeato/mdbook-typst)

## 怎么用

### 下载依赖

#### mdbook

Cargo install安装，_不推荐,速度较慢_

`cargo install mdbook`

`cargo install --git https://github.com/rust-lang/mdBook.git mdbook`

建议到<https://github.com/rust-lang/mdBook/releases>点击下载可执行包，除非没有相应的os版本，不然不推荐构建安装。


#### typst

Typst 是一种基于标记的新型排版系统，其功能与 LaTeX 不相上下，但学习和使用却更加简单。

我们需要将markdown文件转换为typst文件，再借助typst的cli工具进行导出，并且typst也是rust编写。

<https://github.com/typst/typst/releases>

### book.toml配置

在book.toml中加入如下配置,即可生成pdf
```toml
[output.typst-piggsoft]
```

### 可选参数

如下为可选参数以及默认值

```toml
[output.typst-piggsoft]
section_level = 3 #目录最大层级
document_keywords = "keywords" #给pdf的metedata使用
output_format = "pdf" #可选pdf，svg，png
output_dir = "typst-piggsoft" #${book.root} + ${build.build_dir} + ${output_dir}
output_filename = "out" #默认文件名，pdf -> ${output_filename}.pdf; svg -> ${output_filename}-{n}.svg; png -> ${output_filename}-{n}.png.其中{n}为页面的序号，基于SUMMARY.md
template_path = "None" #默认不配置，这是typst相关的前导配置，配置后将读取，${book.root} + ${template_path}
```

## 其他

当前功能还未完全完成，链接和图片还有部分未实现。计划3月完成。

