/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   get_next_line.c                                    :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: jschwabe <jschwabe@student.42.fr>          +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2023/04/18 14:25:50 by jschwabe          #+#    #+#             */
/*   Updated: 2024/05/29 00:22:15 by jschwabe         ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

#include "get_next_line.h"
#include <limits.h>

static char	*read_line(char *buf, int fd, int *buf_idx, char **line);

static char	*check_free(char *buf, int buf_idx, char *line, bool is_buf)
{
	int		buf_nl_idx;
	char	*ret;
	int		gnl_idx;

	if (!line)
		return (NULL);
	if (is_buf)
	{
		buf_nl_idx = index_of(buf, '\n', INT_MAX);
		ft_memcpy(line, buf, buf_idx + 1);
		if (buf[buf_nl_idx] != '\n')
			buf[buf_nl_idx] = 0;
		else
			buf_nl_idx++;
		ft_memcpy(buf, buf + buf_nl_idx, SIZE - buf_nl_idx + 1);
	}
	gnl_idx = index_of(line, '\n', INT_MAX);
	if (line[gnl_idx] == '\n')
		gnl_idx++;
	ret = ft_calloc(sizeof(char), gnl_idx + 1);
	if (!ret)
		return (free(line), NULL);
	ft_memcpy(ret, line, gnl_idx);
	free(line);
	return (ret);
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
		read_line(buf, fd, &buf_idx, &line);
	return (check_free(buf, buf_idx, line, false));
}

static inline bool	iter_line(char **line, char *buf, char *tmp, int buf_idx)
{
	int	buf_nl_idx;

	*line = ft_calloc(sizeof(char), buf_idx + 1);
	if (!*line)
		return (false);
	ft_strlcpy(*line, buf, buf_idx + 1);
	ft_memcpy(buf, tmp, SIZE);
	buf_nl_idx = index_of(buf, '\n', SIZE + 1);
	if (buf[buf_nl_idx] != '\n')
		buf[buf_nl_idx] = 0;
	else
		buf_nl_idx++;
	ft_memcpy(buf, buf + buf_nl_idx, SIZE - buf_nl_idx + 1);
	return (true);
}

static char	*read_line(char *buf, int fd, int *buf_idx, char **line)
{
	char		tmp[SIZE + 1];
	const int	rd = read(fd, ft_memset(tmp, 0, SIZE), SIZE);
	int			tmp_nl_idx;

	if (rd == -1)
		return (ft_memset(buf, 0, SIZE));
	if (rd > 0)
		*buf_idx += SIZE;
	tmp_nl_idx = index_of(tmp, '\n', SIZE);
	if ((tmp[tmp_nl_idx] == '\n' || (rd == 0 && *buf_idx != 0))
		&& !iter_line(line, buf, tmp, *buf_idx))
		return (NULL);
	if (tmp[tmp_nl_idx] != '\n' && rd != 0
		&& !read_line(buf, fd, buf_idx, line))
		return (NULL);
	if (rd > 0 && *buf_idx != 0)
	{
		*buf_idx -= SIZE;
		tmp_nl_idx = index_of(tmp, '\n', SIZE);
		ft_memcpy(*line + *buf_idx, tmp, tmp_nl_idx);
		if (tmp[tmp_nl_idx] == '\n')
			(*line)[*buf_idx + tmp_nl_idx] = '\n';
	}
	return (*line);
}
