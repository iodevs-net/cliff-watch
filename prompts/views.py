from django.shortcuts import render, redirect, get_object_or_404
from django.contrib import messages
from .models import Prompt, Category


def prompt_list(request):
    prompts = Prompt.objects.all()
    categories = Category.objects.all()
    category_filter = request.GET.get("category")
    search_query = request.GET.get("search")

    if category_filter:
        prompts = prompts.filter(category_id=category_filter)

    if search_query:
        prompts = prompts.filter(title__icontains=search_query)

    return render(
        request,
        "prompts/prompt_list.html",
        {
            "prompts": prompts,
            "categories": categories,
            "selected_category": category_filter,
            "search_query": search_query,
        },
    )


def prompt_detail(request, pk):
    prompt = get_object_or_404(Prompt, pk=pk)
    return render(request, "prompts/prompt_detail.html", {"prompt": prompt})


def prompt_create(request):
    if request.method == "POST":
        title = request.POST.get("title")
        content = request.POST.get("content")
        category_id = request.POST.get("category")
        tags = request.POST.get("tags")
        is_favorite = request.POST.get("is_favorite") == "on"

        category = None
        if category_id:
            category = get_object_or_404(Category, pk=category_id)

        Prompt.objects.create(
            title=title,
            content=content,
            category=category,
            tags=tags,
            is_favorite=is_favorite,
        )
        messages.success(request, "Prompt created successfully!")
        return redirect("prompt_list")

    categories = Category.objects.all()
    return render(request, "prompts/prompt_form.html", {"categories": categories})


def prompt_edit(request, pk):
    prompt = get_object_or_404(Prompt, pk=pk)

    if request.method == "POST":
        prompt.title = request.POST.get("title")
        prompt.content = request.POST.get("content")
        category_id = request.POST.get("category")
        prompt.tags = request.POST.get("tags")
        prompt.is_favorite = request.POST.get("is_favorite") == "on"

        if category_id:
            prompt.category = get_object_or_404(Category, pk=category_id)
        else:
            prompt.category = None

        prompt.save()
        messages.success(request, "Prompt updated successfully!")
        return redirect("prompt_detail", pk=prompt.pk)

    categories = Category.objects.all()
    return render(
        request, "prompts/prompt_form.html", {"prompt": prompt, "categories": categories}
    )


def prompt_delete(request, pk):
    prompt = get_object_or_404(Prompt, pk=pk)
    if request.method == "POST":
        prompt.delete()
        messages.success(request, "Prompt deleted successfully!")
        return redirect("prompt_list")
    return render(request, "prompts/prompt_confirm_delete.html", {"prompt": prompt})


def category_list(request):
    categories = Category.objects.all()
    return render(request, "prompts/category_list.html", {"categories": categories})


def category_create(request):
    if request.method == "POST":
        name = request.POST.get("name")
        description = request.POST.get("description")
        Category.objects.create(name=name, description=description)
        messages.success(request, "Category created successfully!")
        return redirect("category_list")
    return render(request, "prompts/category_form.html")


def category_delete(request, pk):
    category = get_object_or_404(Category, pk=pk)
    if request.method == "POST":
        category.delete()
        messages.success(request, "Category deleted successfully!")
        return redirect("category_list")
    return render(request, "prompts/category_confirm_delete.html", {"category": category})
