/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   get_next_line.c                                    :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: jschwabe <jschwabe@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2023/04/18 14:25:50 by jschwabe          #+#    #+#             */
/*   Updated: 2024/05/28 20:36:11 by jschwabe         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "get_next_line.h"
#include <limits.h>

static char	*read_line(char *buf, int fd, int buf_idx, char **line);
int			index_of(char *str, char c, int max_len);

static char	*check_free(char *buf, int buf_idx, char *line, bool is_buf)
{
	int		nl_index;
	char	*tmp;
	int		i;

	if (!line)
		return (NULL);
	if (is_buf)
	{
		nl_index = index_of(buf, '\n', INT_MAX);
		ft_memcpy(line, buf, buf_idx + 1);
		if (buf[nl_index] != '\n')
			buf[nl_index] = '\0';
		else
			nl_index++;
		ft_memcpy(buf, buf + nl_index, SIZE - nl_index + 1);
	}
	i = index_of(line, '\0', INT_MAX);
	tmp = ft_calloc(sizeof(char), i + 1);
	if (!tmp)
		return (free(line), NULL);
	ft_memcpy(tmp, line, i);
	free(line);
	return (tmp);
}

char	*get_next_line(int fd)
{
	char			*line;
	static char		buf[SIZE + 1];
	int				buf_idx;

	if (fd < 0 || SIZE < 1)
		return (NULL);
	line = NULL;
	buf_idx = -1;
	while (++buf_idx < SIZE && buf[buf_idx])
	{
		if (buf[buf_idx] == '\n')
		{
			line = ft_calloc(sizeof(char), SIZE + 1);
			if (!line)
				return (NULL);
			return (check_free(buf, buf_idx, line, true));
		}
	}
	if (buf[buf_idx] != '\n')
		read_line(buf, fd, buf_idx, &line);
	return (check_free(buf, buf_idx, line, false));
}

static char	*read_line(char *buf, int fd, int buf_idx, char **line)
{
	char		tmp[SIZE + 1];
	const int	rd = read(fd, ft_memset(tmp, 0, SIZE), SIZE);
	int			i;

	if (rd == -1)
		return (ft_memset(buf, 0, SIZE));
	if (rd > 0)
		buf_idx += SIZE;
	i = 0;
	while (i < SIZE && tmp[i] != '\n' && tmp[i] != '\0')
		i++;
	if (tmp[i] == '\n' || (rd == 0 && buf_idx != 0))
	{
		*line = ft_calloc(sizeof(char), buf_idx + 1);
		if (!*line)
			return (NULL);
		ft_strlcpy(*line, buf, buf_idx + 1);
		ft_memcpy(buf, tmp, SIZE);
		int	nl_index = index_of(buf, '\n', INT_MAX);
		if (buf[nl_index] != '\n')
			buf[nl_index] = '\0';
		else
			nl_index++;
		ft_memcpy(buf, buf + nl_index, SIZE - nl_index + 1);
	}
	if (tmp[i] != '\n' && rd != 0 && !read_line(buf, fd, buf_idx, line))
		return (NULL);
	else if (rd > 0)
	{
		buf_idx -= SIZE;
		i = 0;
		while (i < SIZE && tmp[i] != '\n' && tmp[i] != '\0')
			i++;
		ft_memcpy(*line + buf_idx, tmp, i);
		if (tmp[i] == '\n')
			(*line)[buf_idx + i] = '\n';
	}
	return (*line);
}
